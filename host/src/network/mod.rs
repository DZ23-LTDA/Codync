//! This computer on codync-network: the user's other computers and the bots on them.
//!
//! The node stays off until the user creates or joins a network. Once it is on, this computer's
//! visible bots are its agents, each under an address name (`reviewer`, so `reviewer@desk` from
//! elsewhere), and the letters that reach them become bot requests (`chat::team::remote`).

mod handles;

use crate::LockExt;
use crate::hub::Hub;
use anyhow::{Context, Result, anyhow, bail};
use codync_network::{Address, Agent, Connector, Node, Peer, Reach, Role, Rule};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub use handles::handle;

const KV_CONNECTOR: &str = "network_connector";
/// Every address name ever given to one of this computer's bots (name → bot id).
const KV_HANDLES: &str = "network_handles";
/// Who on the other computers may write to this computer's bots (`codync_network::Rule`s).
const KV_RULES: &str = "network_rules";
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

#[derive(Default)]
pub struct Network {
    node: OnceLock<Arc<Node>>,
    asked: Mutex<Option<Instant>>,
    handles: Mutex<handles::Handles>,
    /// Names this computer sends letters as without offering them (`Node::set_senders`): the cloud
    /// assistant, and agents that aren't bots (`codync-host mcp network`).
    senders: Mutex<std::collections::BTreeSet<String>>,
}

/// Who a `message_bot` request is for.
pub enum Recipient {
    /// One of this computer's bots, by id (whether it exists is the caller's to check).
    Here(String),
    There(Address),
}

fn dir() -> PathBuf {
    crate::service::data_dir().join("network")
}

impl Network {
    pub fn node(&self) -> Option<Arc<Node>> {
        self.node.get().cloned()
    }

    /// The address name of one of this computer's bots.
    pub fn handle_of(&self, bot: &str) -> Option<String> {
        self.handles.locked().handle(bot)
    }

    /// The other computers and their bots, as last heard; refreshed in the background when stale,
    /// and waited for (briefly) the first time.
    pub async fn directory(&self) -> Vec<Peer> {
        let Some(node) = self.node() else { return Vec::new() };
        match self.due() {
            Some(Due::First) => {
                let _ = tokio::time::timeout(FIRST_LOOK, node.peers()).await;
            }
            Some(Due::Again) => {
                let refresh = node.clone();
                tokio::spawn(async move { refresh.peers().await });
            }
            None => {}
        }
        node.directory().into_iter().filter(|p| !p.me).collect()
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

    /// Where `name` points: a bot id, an address name, `name@` this computer or a bare name none of
    /// its bots has are this computer's; `name@computer`, or a bare name one other computer offers,
    /// is a bot there.
    pub fn recipient(&self, name: &str) -> Result<Recipient> {
        let (handle, computer) = name.rsplit_once('@').map_or((name, None), |(handle, c)| (handle, Some(c)));
        let here = self.handles.locked().bot(handle);
        let local = Ok(Recipient::Here(here.clone().unwrap_or_else(|| name.to_owned())));
        let Some(node) = self.node() else { return local };
        if computer.is_none() && here.is_some() {
            return local;
        }
        match node.resolve(name) {
            Ok(address) if node.name().as_deref() == Some(&address.computer) => local,
            Ok(address) => Ok(Recipient::There(address)),
            Err(_) if computer.is_none() => local,
            Err(error) => Err(error),
        }
    }

    /// Lets this computer send letters as `name` without offering it, unless a bot goes by it.
    pub fn send_as(&self, name: &str) -> Result<()> {
        if let Some(bot) = self.bot_for(name) {
            bail!("a bot here ({bot}) already goes by {name} on the network: give the agent another name");
        }
        let mut senders = self.senders.locked();
        if senders.insert(name.to_owned())
            && let Some(node) = self.node()
        {
            node.set_senders(senders.iter().cloned());
        }
        Ok(())
    }

    /// This computer's bot behind an address name, if any.
    pub fn bot_for(&self, handle: &str) -> Option<String> {
        self.handles.locked().bot(handle)
    }
}

/// Starts the node when this computer is in a network.
pub async fn start(hub: &Arc<Hub>) {
    if !dir().join("network.json").exists() {
        return;
    }
    if let Err(error) = boot(hub).await {
        tracing::warn!(error = format!("{error:#}"), "couldn't start the network");
    }
}

async fn boot(hub: &Arc<Hub>) -> Result<Arc<Node>> {
    if let Some(node) = hub.network.node() {
        return Ok(node);
    }
    let connector: Connector = hub.store.kv_get(KV_CONNECTOR).as_deref().unwrap_or("public").parse()?;
    let (node, inbox) = Node::start(&dir(), &connector).await.context("couldn't start the network")?;
    let rules: Vec<Rule> = hub.store.kv_get(KV_RULES).and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default();
    if let Err(error) = node.set_rules(rules) {
        tracing::warn!(error = format!("{error:#}"), "couldn't apply the network's rules; none apply");
    }
    let node = Arc::new(node);
    hub.network.node.set(node.clone()).map_err(|_| anyhow!("the network is already starting; try again"))?;
    tokio::spawn(watch(hub.clone(), node.changes()));
    publish(hub);
    tokio::spawn(crate::chat::team::remote::listen(hub.clone(), inbox));
    tracing::info!(computer = node.name().as_deref().unwrap_or("(none yet)"), "network on");
    Ok(node)
}

/// Tells this computer's apps and paired phones what the network looks like now, after every
/// change the node reports.
async fn watch(hub: Arc<Hub>, mut changes: tokio::sync::watch::Receiver<()>) {
    while changes.changed().await.is_ok() {
        emit(&hub);
    }
}

/// The `network` event: what `networkView` answers.
pub fn emit(hub: &Hub) {
    let _ = hub.events.send(json!({"type": "network", "network": status(hub)}));
}

/// Offers this computer's visible bots to the network, under their address names. Called after
/// every change to a bot's name, description or visibility.
pub fn publish(hub: &Hub) {
    let Some(node) = hub.network.node() else { return };
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
    node.set_agents(agents);
}

fn joined(hub: &Hub) -> Result<Arc<Node>> {
    hub.network
        .node()
        .filter(|n| n.name().is_some())
        .ok_or_else(|| anyhow!("this computer isn't in a network: create one or join with an invite"))
}

/// `networkStatus`, the `network` event and the stream's hello: this computer and the others, as
/// last heard, with what this computer may do (`canInvite`, `canRecover`) and do to each
/// (`canRemove`, `canPromote`); plus the cloud assistants switch, and whether they can reach this
/// computer at all (`assistantsAvailable`). Never an invite: phones read it too.
pub fn status(hub: &Hub) -> Value {
    let assistants = crate::chat::team::assistant::enabled(&hub.store);
    let available = crate::chat::team::assistant::available(hub);
    let Some(node) = hub.network.node().filter(|n| n.name().is_some()) else {
        return json!({"joined": false, "assistants": assistants, "assistantsAvailable": available});
    };
    let mine = node.role();
    let computers: Vec<Value> = node
        .directory()
        .into_iter()
        .map(|p| {
            let agents: Vec<Value> = p.agents.iter().map(|a| json!({"id": a.id, "name": a.name})).collect();
            let may = |rule: fn(Role, Role) -> bool| !p.me && mine.is_some_and(|me| rule(me, p.role));
            json!({
                "name": p.name, "role": p.role, "me": p.me, "online": p.online, "agents": agents,
                "canRemove": may(Role::removes), "canPromote": may(Role::promotes),
            })
        })
        .collect();
    json!({
        "joined": true, "computer": node.name(), "role": mine,
        "canInvite": mine.is_some_and(Role::invites), "canRecover": mine.is_some_and(|r| r != Role::Owner),
        "connector": hub.store.kv_get(KV_CONNECTOR).unwrap_or_else(|| "public".into()),
        "computers": computers, "assistants": assistants, "assistantsAvailable": available,
        "rules": node.rules(),
    })
}

/// `networkView`: `status`, asking the others in the background when what they offer is stale;
/// the `network` event brings what they answer. The first look since the host started is left to
/// bot listings, which wait for it (`Network::directory`).
pub fn view(hub: &Hub) -> Value {
    let looked = hub.network.asked.locked().is_some();
    if looked
        && let Some(node) = hub.network.node()
        && hub.network.due().is_some()
    {
        tokio::spawn(async move { node.peers().await });
    }
    status(hub)
}

/// `networkCreate`: a new network with this computer as its owner; returns the recovery code.
pub async fn create(hub: &Arc<Hub>, name: Option<&str>, connector: Option<&str>) -> Result<Value> {
    if joined(hub).is_ok() {
        bail!("this computer is already in a network");
    }
    set_connector(hub, connector)?;
    let node = boot(hub).await?;
    let code = node.create(&name.map_or_else(default_name, str::to_owned))?;
    publish(hub);
    Ok(json!({"computer": node.name(), "recoveryCode": code}))
}

/// `networkJoin`: joins with an invite from the owner or an admin.
pub async fn join(hub: &Arc<Hub>, invite: &str, name: Option<&str>, connector: Option<&str>) -> Result<Value> {
    if joined(hub).is_ok() {
        bail!("this computer is already in a network");
    }
    set_connector(hub, connector)?;
    let node = boot(hub).await?;
    node.join(invite, &name.map_or_else(default_name, str::to_owned)).await?;
    publish(hub);
    Ok(status(hub))
}

/// `networkInvite`: a one-time invite and when it stops working (Unix ms).
pub fn invite(hub: &Hub) -> Result<Value> {
    let invite = joined(hub)?.invite()?;
    let lasts = i64::try_from(codync_network::INVITE_FOR.as_millis()).expect("an invite lasts an hour");
    let until = crate::store::now_ms().saturating_add(lasts);
    Ok(json!({"invite": invite, "expiresAt": until}))
}

/// `networkPeers`: asks every computer now.
pub async fn peers(hub: &Arc<Hub>) -> Result<Value> {
    joined(hub)?.peers().await?;
    *hub.network.asked.locked() = Some(Instant::now());
    Ok(status(hub))
}

pub fn remove(hub: &Hub, computer: &str, cascade: bool) -> Result<Value> {
    joined(hub)?.remove(computer, cascade)?;
    Ok(json!({}))
}

/// `setNetworkRules`: who on the other computers may write to this computer's bots, in order
/// (the first match decides); an empty list lets everyone.
pub fn set_rules(hub: &Hub, rules: &Value) -> Result<Value> {
    let rules: Vec<Rule> =
        serde_json::from_value(rules.clone()).context("rules are {fromComputer, fromAgent, toAgent, access}")?;
    joined(hub)?.set_rules(rules.clone())?;
    hub.store.kv_set(KV_RULES, &json!(rules).to_string())?;
    emit(hub);
    Ok(status(hub))
}

pub fn promote(hub: &Hub, computer: &str) -> Result<Value> {
    joined(hub)?.promote(computer)?;
    Ok(json!({}))
}

pub fn recover(hub: &Hub, code: &str) -> Result<Value> {
    joined(hub)?.recover(code)?;
    Ok(json!({}))
}

fn set_connector(hub: &Hub, connector: Option<&str>) -> Result<()> {
    if let Some(connector) = connector {
        connector.parse::<Connector>()?;
        hub.store.kv_set(KV_CONNECTOR, connector)?;
    }
    Ok(())
}

/// This computer's host name, as a network name can hold it.
fn default_name() -> String {
    let host = gethostname::gethostname().to_string_lossy().into_owned();
    handle(host.split('.').next().unwrap_or("computer"))
}
