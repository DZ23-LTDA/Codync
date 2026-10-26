//! This computer on codync-network: the user's other computers and the bots on them.
//!
//! A computer can be in several networks (`home`, `work`). Each one has its own node, with its
//! own key, roster, connector and rules, in its own folder, so the networks stay apart: members of
//! one can't tell this computer is in another, and nothing one network decides reaches the
//! others. No node runs until the user creates or joins a network. This computer's visible bots
//! are offered in every network, each under one address name (`reviewer`, so `reviewer@desk` from
//! elsewhere, or `reviewer@desk.home` to name the network), and the letters that reach them become
//! bot requests (`chat::team::remote`).

mod api;
mod handles;
mod saved;

use crate::LockExt;
use crate::hub::Hub;
use anyhow::{Context, Result, anyhow, bail};
use codync_network::{Address, Agent, Connector, Node, Peer, Reach};
use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub use api::{create, emit, invite, join, leave, peers, promote, recover, remove, set_rules, status, view};
pub use handles::handle;

/// Every address name ever given to one of this computer's bots (name → bot id).
const KV_HANDLES: &str = "network_handles";
/// How long what the other computers offer is good for before a listing asks them again.
const FRESH_FOR: Duration = Duration::from_mins(1);
/// How long a listing waits for computers it has never heard from.
const FIRST_LOOK: Duration = Duration::from_secs(5);

/// Why what the other computers offer is to be asked again.
enum Due {
    /// Never asked since the host started: a listing waits for the answers.
    First,
    /// Asked more than `FRESH_FOR` ago.
    Again,
}

/// One network this computer is in.
#[derive(Clone)]
pub struct Slot {
    /// Its folder in the data directory, and its id in the API.
    pub id: String,
    /// What the user calls it (`home`), unique on this computer.
    pub name: String,
    pub node: Arc<Node>,
}

#[derive(Default)]
pub struct Network {
    /// The first is the default: bare names look there first.
    slots: Mutex<Vec<Slot>>,
    asked: Mutex<Option<Instant>>,
    handles: Mutex<handles::Handles>,
    /// Names this computer sends letters as without offering them (`Node::set_senders`): the cloud
    /// assistant, and agents that aren't bots (`codync-host mcp network`).
    senders: Mutex<BTreeSet<String>>,
    /// One create, join or leave at a time.
    changing: tokio::sync::Mutex<()>,
}

/// Who a `message_bot` request is for.
pub enum Recipient {
    /// One of this computer's bots, by id (whether it exists is the caller's to check).
    Here(String),
    There(Target),
}

/// A bot on another computer, and the network the letter goes through.
#[derive(Clone)]
pub struct Target {
    pub network: String,
    pub address: Address,
}

impl Network {
    pub fn slots(&self) -> Vec<Slot> {
        self.slots.locked().clone()
    }

    /// A network by its id or its name; the only one when `which` is `None`.
    pub fn slot(&self, which: Option<&str>) -> Result<Slot> {
        let slots = self.slots.locked();
        match which {
            Some(w) => slots.iter().find(|s| s.id == w || s.name.eq_ignore_ascii_case(w)).cloned().ok_or_else(|| {
                let names: Vec<&str> = slots.iter().map(|s| s.name.as_str()).collect();
                anyhow!("no network called {w} here (this computer is in: {})", list(&names))
            }),
            None => match &slots[..] {
                [only] => Ok(only.clone()),
                [] => bail!("this computer isn't in a network: create one or join with an invite"),
                many => {
                    let names: Vec<&str> = many.iter().map(|s| s.name.as_str()).collect();
                    bail!("this computer is in several networks ({}): say which", list(&names))
                }
            },
        }
    }

    /// The address name of one of this computer's bots.
    pub fn handle_of(&self, bot: &str) -> Option<String> {
        self.handles.locked().handle(bot)
    }

    /// This computer's name in its first network, if it is in one.
    pub fn my_name(&self) -> Option<String> {
        self.slots.locked().first().and_then(|s| s.node.name())
    }

    /// The other computers and their bots in every network, as last heard; refreshed in the
    /// background when stale, and waited for (briefly) the first time. A computer in several
    /// networks names each of their computers with the network (`desk.home`).
    pub async fn directory(&self) -> Vec<Peer> {
        let slots = self.slots();
        match self.due() {
            Some(Due::First) => {
                let asks = slots.iter().map(|s| s.node.peers());
                let _ = tokio::time::timeout(FIRST_LOOK, futures::future::join_all(asks)).await;
            }
            Some(Due::Again) => {
                for slot in &slots {
                    let node = slot.node.clone();
                    tokio::spawn(async move { node.peers().await });
                }
            }
            None => {}
        }
        let several = slots.len() > 1;
        slots
            .iter()
            .flat_map(|s| {
                s.node.directory().into_iter().filter(|p| !p.me).map(move |mut p| {
                    if several {
                        p.name = format!("{}.{}", p.name, s.name);
                    }
                    p
                })
            })
            .collect()
    }

    /// Whether what the other computers offer is due to be asked again; counted as asked from now.
    fn due(&self) -> Option<Due> {
        let mut asked = self.asked.locked();
        let due = match *asked {
            None => Due::First,
            Some(at) if at.elapsed() >= FRESH_FOR => Due::Again,
            Some(_) => return None,
        };
        *asked = Some(Instant::now());
        Some(due)
    }

    /// Where `name` points. A bot id or address name of this computer's is here, and so is a bare
    /// name no network offers. `name@computer` is that computer in whichever network has it, and
    /// `name@computer.network` names the network; a bare name one other computer offers is there.
    /// Refuses a name that two networks would both take, naming the full addresses.
    pub fn recipient(&self, name: &str) -> Result<Recipient> {
        let (handle, at) = name.rsplit_once('@').map_or((name, None), |(handle, at)| (handle, Some(at)));
        let here = self.handles.locked().bot(handle);
        let local = || Ok(Recipient::Here(here.clone().unwrap_or_else(|| name.to_owned())));
        if at.is_none() && here.is_some() {
            return local();
        }
        let (computer, network) = match at.map(|at| at.split_once('.').map_or((at, None), |(c, n)| (c, Some(n)))) {
            Some((computer, network)) => (Some(computer), network),
            None => (None, None),
        };
        let slots = match network {
            Some(n) => vec![self.slot(Some(n))?],
            None => self.slots(),
        };
        let mut found: Vec<Target> = Vec::new();
        let mut errors = Vec::new();
        for slot in &slots {
            let ask = computer.map_or_else(|| handle.to_owned(), |c| format!("{handle}@{c}"));
            match slot.node.resolve(&ask) {
                Ok(address) if slot.node.name().as_deref() == Some(&address.computer) => return local(),
                Ok(address) => found.push(Target { network: slot.id.clone(), address }),
                Err(error) => errors.push(error),
            }
        }
        match &found[..] {
            [] if computer.is_none() => local(),
            [] => Err(errors.into_iter().next().unwrap_or_else(|| anyhow!("this computer isn't in a network"))),
            [only] => Ok(Recipient::There(only.clone())),
            many => {
                let names: Vec<String> = many.iter().map(|t| self.full_address(t)).collect();
                bail!("more than one network has {name}: {}; use the full address", list(&names))
            }
        }
    }

    /// How a target reads in listings and chats: `agent@computer`, with the network
    /// (`agent@computer.home`) while this computer is in several.
    pub fn address_text(&self, target: &Target) -> String {
        if self.slots.locked().len() > 1 { self.full_address(target) } else { target.address.to_string() }
    }

    /// `agent@computer.network`.
    fn full_address(&self, target: &Target) -> String {
        let network = self.slot(Some(&target.network)).map_or_else(|_| target.network.clone(), |s| s.name);
        format!("{}.{network}", target.address)
    }

    /// Lets this computer send letters as `name` without offering it, unless a bot goes by it.
    pub fn send_as(&self, name: &str) -> Result<()> {
        if let Some(bot) = self.bot_for(name) {
            bail!("a bot here ({bot}) already goes by {name} on the network: give the agent another name");
        }
        let mut senders = self.senders.locked();
        if senders.insert(name.to_owned()) {
            for slot in self.slots() {
                slot.node.set_senders(senders.iter().cloned());
            }
        }
        Ok(())
    }

    /// This computer's bot behind an address name, if any.
    pub fn bot_for(&self, handle: &str) -> Option<String> {
        self.handles.locked().bot(handle)
    }
}

/// `a, b and c`.
fn list(names: &[impl AsRef<str>]) -> String {
    let names: Vec<&str> = names.iter().map(AsRef::as_ref).collect();
    match &names[..] {
        [] => "none".into(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// Starts the node of every network this computer is in.
pub async fn start(hub: &Arc<Hub>) {
    for saved in saved::load(hub) {
        let id = saved.id.clone();
        if let Err(error) = boot(hub, &saved).await {
            tracing::warn!(network = id, error = format!("{error:#}"), "couldn't start a network");
        }
    }
}

/// Starts one network's node and takes it in.
async fn boot(hub: &Arc<Hub>, saved: &saved::Saved) -> Result<Slot> {
    let (node, inbox) = start_node(saved).await?;
    Ok(take_in(hub, saved, node, inbox))
}

async fn start_node(saved: &saved::Saved) -> Result<(Node, tokio::sync::mpsc::Receiver<codync_network::Message>)> {
    let connector: Connector = saved.connector.parse()?;
    Node::start(&saved::dir(&saved.id), &connector).await.context("couldn't start the network")
}

/// A started node becomes one of this computer's networks: its rules and name, what this computer
/// offers, and the letters that reach it.
fn take_in(
    hub: &Arc<Hub>,
    saved: &saved::Saved,
    node: Node,
    inbox: tokio::sync::mpsc::Receiver<codync_network::Message>,
) -> Slot {
    if let Err(error) = node.set_rules(saved.rules.clone()) {
        tracing::warn!(error = format!("{error:#}"), "couldn't apply the network's rules; none apply");
    }
    node.set_network_name(Some(saved.name.clone()));
    node.set_senders(hub.network.senders.locked().iter().cloned());
    let slot = Slot { id: saved.id.clone(), name: saved.name.clone(), node: Arc::new(node) };
    hub.network.slots.locked().push(slot.clone());
    tokio::spawn(watch(hub.clone(), slot.node.changes()));
    publish(hub);
    tokio::spawn(crate::chat::team::remote::listen(hub.clone(), slot.id.clone(), inbox));
    tracing::info!(network = slot.name, computer = slot.node.name().as_deref().unwrap_or("(none yet)"), "network on");
    slot
}

/// Tells this computer's apps and paired phones what the networks look like now, after every
/// change a node reports. Ends when the node goes (leaving the network).
async fn watch(hub: Arc<Hub>, mut changes: tokio::sync::watch::Receiver<()>) {
    while changes.changed().await.is_ok() {
        emit(&hub);
    }
}

/// Offers this computer's visible bots to every network, under their address names. Called
/// after every change to a bot's name, description or visibility.
pub fn publish(hub: &Hub) {
    let slots = hub.network.slots();
    if slots.is_empty() {
        return;
    }
    let bots = match crate::chat::team::visible_bots(&hub.store) {
        Ok(bots) => bots,
        Err(error) => {
            tracing::warn!(error = format!("{error:#}"), "couldn't offer the bots to the network");
            return;
        }
    };
    let mut handles = hub.network.handles.locked();
    let mut given: HashMap<String, String> =
        hub.store.kv_get(KV_HANDLES).and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default();
    // The cloud assistant's address name never goes to a bot (`chat::team::assistant::HANDLE`).
    given.entry(crate::chat::team::assistant::HANDLE.into()).or_insert_with(|| "assistant".into());
    let before = given.len();
    *handles = handles::Handles::of(bots.iter().map(|b| (b.id.clone(), b.name.clone())), &mut given);
    if given.len() != before
        && let Err(error) = hub.store.kv_set(KV_HANDLES, &json!(given).to_string())
    {
        tracing::warn!(error = format!("{error:#}"), "couldn't save the bots' address names");
    }
    let agents: Vec<(Agent, Reach)> = bots
        .iter()
        .filter(|b| b.offered_to_network)
        .filter_map(|b| {
            let id = handles.handle(&b.id)?;
            Some((Agent { id, name: b.name.clone(), about: b.description.clone() }, Reach::App))
        })
        .collect();
    drop(handles);
    for slot in slots {
        slot.node.set_agents(agents.clone());
    }
}

/// This computer's host name, as a network name can hold it.
fn default_name() -> String {
    let host = gethostname::gethostname().to_string_lossy().into_owned();
    handle(host.split('.').next().unwrap_or("computer"))
}
