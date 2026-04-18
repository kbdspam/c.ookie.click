use std::time::Duration;

use axum::{
	extract::{
		WebSocketUpgrade,
		ws::{Message, WebSocket},
	},
	response::Response,
};
use futures::{SinkExt, StreamExt};

pub(super) async fn handler(ws: WebSocketUpgrade) -> Response {
	ws.max_message_size(400).max_frame_size(500).on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
	let (mut send, mut recv) = socket.split();

	// axum automagically handles ping/pongs for us

	let mut tasks = tokio::task::JoinSet::new();

	tasks.spawn(async move {
		while let Ok(_) = send.send(axum::extract::ws::Message::Text(":3".into())).await {
			tokio::time::sleep(Duration::from_mins(1)).await;
		}
	});
	tasks.spawn(async move {
		while let Some(Ok(msg)) = recv.next().await {
			if let Message::Close(_) = msg {
				break;
			}
		}
	});

	while let Some(_) = tasks.join_next().await {
		tasks.abort_all();
	}
}
