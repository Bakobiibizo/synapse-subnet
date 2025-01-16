//! WebSocket handlers for real-time updates

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::interface::core::error::{Error, Result, retry, RetryConfig};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Status update
    Status(crate::interface::core::models::Status),
    /// Metrics update
    Metrics(crate::interface::core::models::Metrics),
    /// Module update
    Module(crate::interface::core::models::ModuleUpdate),
    /// Chain event
    ChainEvent(crate::interface::core::models::ChainEvent),
    /// Network update
    NetworkUpdate(crate::interface::core::models::NetworkStatus),
    /// Priority update
    PriorityUpdate(crate::interface::core::models::PriorityLevel),
    /// Resource usage
    ResourceUsage(crate::interface::core::models::ResourceMetrics),
    /// Stake update
    StakeUpdate(crate::interface::core::models::StakeInfo),
    /// Error message
    Error(String),
}

/// WebSocket state
#[derive(Clone)]
pub struct WsState {
    /// Message sender
    pub tx: broadcast::Sender<WsMessage>,
    /// Connected clients
    pub clients: Arc<Mutex<Vec<broadcast::Sender<WsMessage>>>>,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

impl WsState {
    /// Create new WebSocket state
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            clients: Arc::new(Mutex::new(Vec::new())),
            retry_config: RetryConfig::default(),
        }
    }

    /// Broadcast message to all clients with retry
    pub async fn broadcast(&self, message: WsMessage) -> Result<()> {
        let tx = self.tx.clone();
        retry(|| async move {
            tx.send(message.clone())
                .map_err(|e| Error::WebSocket(e.to_string()))?;
            Ok(())
        }, self.retry_config.clone()).await
    }

    /// Send message to specific client with retry
    pub async fn send_to(&self, client_id: usize, message: WsMessage) -> Result<()> {
        let clients = self.clients.lock().await;
        if let Some(tx) = clients.get(client_id) {
            let tx = tx.clone();
            retry(|| async move {
                tx.send(message.clone())
                    .map_err(|e| Error::WebSocket(e.to_string()))?;
                Ok(())
            }, self.retry_config.clone()).await
        } else {
            Err(Error::WebSocket("Client not found".into()))
        }
    }
}

/// Handle WebSocket connection upgrade
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send initial state
    if let Ok(initial_state) = get_initial_state(&state).await {
        if let Ok(msg) = serde_json::to_string(&initial_state) {
            let _ = sender.send(Message::Text(msg)).await;
        }
    }

    // Handle incoming messages with retry
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let retry_config = RetryConfig {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 1000,
                backoff_multiplier: 2.0,
            };

            let result = retry(|| async {
                if let Ok(msg) = serde_json::to_string(&msg) {
                    sender.send(Message::Text(msg))
                        .await
                        .map_err(|e| Error::WebSocket(e.to_string()))?;
                }
                Ok(())
            }, retry_config).await;

            if result.is_err() {
                break;
            }
        }
    });

    // Handle outgoing messages
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                        let _ = state_clone.broadcast(msg).await;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

/// Get initial state for new connections
async fn get_initial_state(state: &WsState) -> Result<WsMessage> {
    // Collect initial state with retry
    retry(|| async {
        let metrics = get_latest_metrics().await?;
        let status = get_latest_status().await?;
        let network = get_network_status().await?;
        
        Ok(WsMessage::Status(status))
    }, state.retry_config.clone()).await
}

/// Get latest metrics with retry
async fn get_latest_metrics() -> Result<crate::interface::core::models::Metrics> {
    // Implementation with retry mechanism
    todo!()
}

/// Get latest status with retry
async fn get_latest_status() -> Result<crate::interface::core::models::Status> {
    // Implementation with retry mechanism
    todo!()
}

/// Get network status with retry
async fn get_network_status() -> Result<crate::interface::core::models::NetworkStatus> {
    // Implementation with retry mechanism
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message as TMessage;

    /// Test WebSocket connection
    #[tokio::test]
    async fn test_ws_connection() {
        // Start test server
        let state = WsState::new();
        let app = axum::Router::new()
            .route("/ws", axum::routing::get(ws_handler))
            .with_state(state.clone());

        let server = axum::Server::bind(&"127.0.0.1:0".parse().unwrap())
            .serve(app.into_make_service());
        let addr = server.local_addr();
        tokio::spawn(server);

        // Connect test client
        let url = format!("ws://{}/ws", addr);
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        // Send test message
        let test_msg = WsMessage::Status(crate::interface::core::models::Status {
            is_active: true,
            timestamp: chrono::Utc::now(),
        });
        let msg = serde_json::to_string(&test_msg).unwrap();
        write.send(TMessage::Text(msg)).await.unwrap();

        // Verify echo
        if let Some(Ok(TMessage::Text(received))) = read.next().await {
            let received_msg: WsMessage = serde_json::from_str(&received).unwrap();
            match received_msg {
                WsMessage::Status(status) => {
                    assert!(status.is_active);
                }
                _ => panic!("Unexpected message type"),
            }
        }
    }
}
