use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::auth::keys::hash_api_key;
use crate::state::AppState;
use crate::ws::{ClientMessage, MemoryEvent, ServerMessage, Subscription};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// GET /v1/ws?token=amr_sk_... — WebSocket upgrade with auth.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Response {
    let token = match query.token {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "missing token query parameter").into_response();
        }
    };

    if !token.starts_with("amr_sk_") {
        return (StatusCode::UNAUTHORIZED, "invalid key format").into_response();
    }

    let key_hash = hash_api_key(&token);

    let row: Option<AuthRow> = sqlx::query_as(
        "SELECT ak.tenant_id, t.plan FROM api_keys ak JOIN tenants t ON t.id = ak.tenant_id WHERE ak.key_hash = $1 AND ak.revoked_at IS NULL AND (ak.expires_at IS NULL OR ak.expires_at > now())"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let row = match row {
        Some(r) => r,
        None => {
            return (StatusCode::UNAUTHORIZED, "invalid API key").into_response();
        }
    };

    let tenant_id = row.tenant_id;
    ws.on_upgrade(move |socket| handle_socket(socket, state, tenant_id))
}

#[derive(Debug, sqlx::FromRow)]
struct AuthRow {
    tenant_id: Uuid,
    #[allow(dead_code)]
    plan: String,
}

/// Outbound messages to send to the client (either broadcast events or direct responses).
enum Outbound {
    Event(MemoryEvent),
    Direct(String),
}

async fn handle_socket(socket: WebSocket, state: AppState, tenant_id: Uuid) {
    let ws_manager = &state.ws;
    let conn_id = ws_manager.register(tenant_id, None).await;
    let mut broadcast_rx = ws_manager.tx.subscribe();

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Channel for direct responses (presence, pong, errors).
    let (direct_tx, mut direct_rx) = mpsc::channel::<String>(32);

    // Task: send outbound messages to the WS client.
    let ws_mgr_send = ws_manager.clone();
    let db_send = state.db.clone();
    let send_task = tokio::spawn(async move {
        loop {
            let outbound = tokio::select! {
                msg = broadcast_rx.recv() => {
                    match msg {
                        Ok(event) => Some(Outbound::Event(event)),
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("ws conn {} lagged by {} messages", conn_id, n);
                            None
                        }
                        Err(broadcast::error::RecvError::Closed) => return,
                    }
                }
                msg = direct_rx.recv() => {
                    match msg {
                        Some(text) => Some(Outbound::Direct(text)),
                        None => return,
                    }
                }
            };

            let json = match outbound {
                Some(Outbound::Direct(text)) => text,
                Some(Outbound::Event(event)) => {
                    if event.tenant_id != tenant_id {
                        continue;
                    }

                    // Check subscription match.
                    let subscribed = {
                        let clients = ws_mgr_send.clients.read().await;
                        match clients.get(&conn_id) {
                            Some(client) if !client.subscriptions.is_empty() => {
                                client.subscriptions.iter().any(|sub| {
                                    let ns_match = sub.namespace.is_none()
                                        || sub.namespace.as_deref() == event.namespace.as_deref();
                                    let agent_match = sub.agent_id.is_none()
                                        || sub.agent_id.as_deref() == event.agent_id.as_deref();
                                    ns_match && agent_match
                                })
                            }
                            _ => false,
                        }
                    };

                    if !subscribed {
                        continue;
                    }

                    // Check ACL permission.
                    if !check_event_permission(&db_send, tenant_id, &ws_mgr_send, conn_id, &event)
                        .await
                    {
                        continue;
                    }

                    match serde_json::to_string(&event) {
                        Ok(j) => j,
                        Err(_) => continue,
                    }
                }
                None => continue,
            };

            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                return;
            }
        }
    });

    // Read messages from client.
    let ws_mgr_read = ws_manager.clone();
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let parsed: Result<ClientMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(ClientMessage::Subscribe {
                        namespace,
                        agent_id,
                    }) => {
                        if let Some(ref aid) = agent_id {
                            if let Some(client) =
                                ws_mgr_read.clients.write().await.get_mut(&conn_id)
                            {
                                client.agent_id = Some(aid.clone());
                            }
                        }
                        ws_mgr_read
                            .subscribe(
                                conn_id,
                                Subscription {
                                    namespace,
                                    agent_id,
                                },
                            )
                            .await;
                    }
                    Ok(ClientMessage::Unsubscribe { namespace }) => {
                        ws_mgr_read.unsubscribe(conn_id, namespace.as_deref()).await;
                    }
                    Ok(ClientMessage::Presence { .. }) => {
                        let agents = ws_mgr_read.presence(tenant_id).await;
                        let msg = ServerMessage::PresenceUpdate { agents };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = direct_tx.send(json).await;
                        }
                    }
                    Ok(ClientMessage::Ping {}) => {
                        let msg = ServerMessage::Pong {};
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = direct_tx.send(json).await;
                        }
                    }
                    Err(e) => {
                        tracing::debug!("invalid ws message: {}", e);
                        let msg = ServerMessage::Error {
                            message: format!("invalid message: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = direct_tx.send(json).await;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup.
    ws_manager.unregister(conn_id).await;
    send_task.abort();
}

/// Check if the connected client has permission to see this event.
async fn check_event_permission(
    db: &PgPool,
    tenant_id: Uuid,
    ws_mgr: &crate::ws::WsManager,
    conn_id: Uuid,
    event: &MemoryEvent,
) -> bool {
    let client_agent_id = {
        let clients = ws_mgr.clients.read().await;
        match clients.get(&conn_id) {
            Some(c) => c.agent_id.clone(),
            None => return false,
        }
    };

    let event_agent = event.agent_id.as_deref().unwrap_or("default");

    // Owner always sees their own events.
    if client_agent_id.as_deref() == Some(event_agent) {
        return true;
    }

    // No agent_id set = tenant-wide listener, allow all tenant events.
    let client_agent = match client_agent_id {
        Some(ref a) => a.as_str(),
        None => return true,
    };

    let event_ns = event.namespace.as_deref().unwrap_or("default");

    let has_share: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM memory_shares WHERE tenant_id = $1 AND agent_id = $2 AND target_agent_id = $3 AND (namespace = $4 OR namespace = '*') LIMIT 1"
    )
    .bind(tenant_id)
    .bind(event_agent)
    .bind(client_agent)
    .bind(event_ns)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    has_share.is_some()
}
