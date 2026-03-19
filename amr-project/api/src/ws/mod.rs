use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Event broadcast through the pub/sub channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Tenant that owns this event (for filtering).
    pub tenant_id: Uuid,
}

/// A single client's subscription filters.
#[derive(Debug, Clone)]
pub struct Subscription {
    pub namespace: Option<String>, // None = wildcard
    pub agent_id: Option<String>,  // None = wildcard
}

/// Info about a connected WebSocket client.
#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub tenant_id: Uuid,
    pub agent_id: Option<String>,
    pub subscriptions: Vec<Subscription>,
}

/// Manages connected WS clients and presence.
#[derive(Clone)]
pub struct WsManager {
    /// broadcast sender for memory events
    pub tx: broadcast::Sender<MemoryEvent>,
    /// connected clients keyed by a unique connection id
    pub clients: Arc<RwLock<HashMap<Uuid, ConnectedClient>>>,
}

impl WsManager {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new WS connection. Returns a connection id.
    pub async fn register(&self, tenant_id: Uuid, agent_id: Option<String>) -> Uuid {
        let conn_id = Uuid::new_v4();
        let client = ConnectedClient {
            tenant_id,
            agent_id,
            subscriptions: Vec::new(),
        };
        self.clients.write().await.insert(conn_id, client);
        conn_id
    }

    /// Remove a connection.
    pub async fn unregister(&self, conn_id: Uuid) {
        self.clients.write().await.remove(&conn_id);
    }

    /// Add a subscription for a connection.
    pub async fn subscribe(&self, conn_id: Uuid, sub: Subscription) {
        if let Some(client) = self.clients.write().await.get_mut(&conn_id) {
            client.subscriptions.push(sub);
        }
    }

    /// Remove subscriptions matching namespace for a connection.
    pub async fn unsubscribe(&self, conn_id: Uuid, namespace: Option<&str>) {
        if let Some(client) = self.clients.write().await.get_mut(&conn_id) {
            client.subscriptions.retain(|s| s.namespace.as_deref() != namespace);
        }
    }

    /// Get list of currently connected agent ids for a tenant.
    pub async fn presence(&self, tenant_id: Uuid) -> Vec<String> {
        let clients = self.clients.read().await;
        let mut agents: HashSet<String> = HashSet::new();
        for client in clients.values() {
            if client.tenant_id == tenant_id {
                if let Some(ref aid) = client.agent_id {
                    agents.insert(aid.clone());
                }
            }
        }
        agents.into_iter().collect()
    }

    /// Broadcast an event.
    pub fn broadcast(&self, event: MemoryEvent) {
        // Ignore send errors (no receivers).
        let _ = self.tx.send(event);
    }
}

/// Client-to-server message.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Subscribe {
        namespace: Option<String>,
        agent_id: Option<String>,
    },
    Unsubscribe {
        namespace: Option<String>,
    },
    Presence {
        #[serde(default)]
        agent_id: Option<String>,
    },
    Ping {},
}

/// Server-to-client message.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    #[serde(rename = "presence.update")]
    PresenceUpdate { agents: Vec<String> },
    Pong {},
    Error { message: String },
}
