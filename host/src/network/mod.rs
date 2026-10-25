//! This computer on codync-network: the user's other computers and the bots on them.
//!
//! The node stays off until the user creates or joins a network. Once it is on, this computer's
//! visible bots are its agents, each under an address name (`reviewer`, so `reviewer@desk` from
//! elsewhere), and the letters that reach them become bot requests (`chat::team::remote`).

mod handles;

use crate::LockExt;
use crate::hub::Hub;
use anyhow::{Context, Result, anyhow, bail};
use codync_network::{Address, Agent, Connector, Node, Peer, Reach};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

pub use handles::handle;

const KV_CONNECTOR: &str = "network_connector";
/// How long what the other computers offer is good for before a listing asks them again.
const FRESH_FOR: Duration = Duration::from_mins(1);
/// How long a listing waits for computers it has never heard from.
const FIRST_LOOK: Duration = Duration::from_secs(5);

/// A request one of this computer's bots sent to another computer, until its reply comes.
pub struct Pending {
    pub sender: String,
    /// The sender's exchange notice.
    pub entry: String,
    pub quote: String,
    pub peer: String,
}

#[derive(Default)]
pub struct Network {
    node: RwLock<Option<Arc<Node>>>,
    /// Outgoing requests by letter id.
    pub pending: Mutex<std::collections::HashMap<String, Pending>>,
    asked: Mutex<Option<Instant>>,
    handles: Mutex<handles::Handles>,
}

fn dir() -> PathBuf {
    crate::service::data_dir().join("network")
}

impl Network {
    pub fn node(&self) -> Option<Arc<Node>> {
        self.node.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    /// This computer's bot behind an address name, if any.
    pub fn bot_for(&self, handle: &str) -> Option<String> {
        self.handles.locked().bot(handle)
    }

    /// The address name of one of this computer's bots.
    pub fn handle_of(&self, bot: &str) -> Option<String> {
        self.handles.locked().handle(bot)
    }

    /// The other computers and their bots, as last heard; refreshed in the background when stale,
    /// and waited for (briefly) the first time.
    pub async fn directory(&self) -> Vec<Peer> {
        let Some(node) = self.node() else { return Vec::new() };
        let asked = *self.asked.locked();
        match asked {
            None => {
                *self.asked.locked() = Some(Instant::now());
                let _ = tokio::time::timeout(FIRST_LOOK, node.peers()).await;
            }
            Some(at) if at.elapsed() >= FRESH_FOR => {
                *self.asked.locked() = Some(Instant::now());
                let refresh = node.clone();
                tokio::spawn(async move { refresh.peers().await });
            }
            Some(_) => {}
        }
        node.directory().into_iter().filter(|p| !p.me).collect()
    }

    /// Where a bot name points when it isn't one of this computer's: `name@computer`, or a bare
    /// name exactly one other computer offers.
    pub fn remote(&self, name: &str) -> Option<Result<Address>> {
        let node = self.node()?;
        if !name.contains('@') && self.bot_for(name).is_some() {
            return None;
        }
        let address = node.resolve(name);
        match &address {
            Ok(a) if node.name().as_deref() == Some(&a.computer) => None,
            Err(_) if !name.contains('@') => None,
            _ => Some(address),
        }
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
    let node = Arc::new(node);
    *hub.network.node.write().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(node.clone());
    publish(hub);
    tokio::spawn(crate::chat::team::remote::listen(hub.clone(), inbox));
    tracing::info!(computer = node.name().as_deref().unwrap_or("(none yet)"), "network on");
    Ok(node)
}

/// Offers this computer's visible bots to the network, under their address names.
pub fn publish(hub: &Hub) {
    let Some(node) = hub.network.node() else { return };
    let bots: Vec<_> = hub
        .store
        .bots()
        .unwrap_or_default()
        .into_iter()
        .filter(|b| !b.deleted && !b.config.hidden && !b.config.is_group())
        .map(|b| b.config)
        .collect();
    let handles = handles::Handles::of(bots.iter().map(|b| (b.id.clone(), b.name.clone())));
    let agents: Vec<(Agent, Reach)> = bots
        .iter()
        .filter_map(|b| {
            let id = handles.handle(&b.id)?;
            Some((Agent { id, name: b.name.clone(), about: b.description.clone() }, Reach::App))
        })
        .collect();
    *hub.network.handles.locked() = handles;
    node.set_agents(agents);
}

fn joined(hub: &Hub) -> Result<Arc<Node>> {
    hub.network
        .node()
        .filter(|n| n.name().is_some())
        .ok_or_else(|| anyhow!("this computer isn't in a network: create one or join with an invite"))
}

/// `networkStatus`: this computer and the others, as last heard.
pub fn status(hub: &Hub) -> Value {
    let Some(node) = hub.network.node().filter(|n| n.name().is_some()) else {
        return json!({"joined": false});
    };
    let computers: Vec<Value> = node
        .directory()
        .into_iter()
        .map(|p| {
            let bots: Vec<&str> = p.agents.iter().map(|a| a.id.as_str()).collect();
            json!({"name": p.name, "role": p.role, "me": p.me, "online": p.online, "bots": bots})
        })
        .collect();
    json!({
        "joined": true, "computer": node.name(), "role": node.role(),
        "connector": hub.store.kv_get(KV_CONNECTOR).unwrap_or_else(|| "public".into()),
        "computers": computers,
    })
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

pub fn invite(hub: &Hub) -> Result<Value> {
    Ok(json!({"invite": joined(hub)?.invite()?}))
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
