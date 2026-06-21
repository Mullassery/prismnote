use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use crate::AppState;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

pub async fn notebook_ws(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let rx = state.stream_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

/// Forward live cell-output stream messages to the client, while draining (and
/// ignoring) anything the client sends, until either side closes.
async fn handle_socket(
    socket: WebSocket,
    mut rx: tokio::sync::broadcast::Receiver<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {}
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
