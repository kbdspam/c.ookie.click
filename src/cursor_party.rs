// SPDX-License-Identifier:
// Copyright

use axum::{
	Router,
	body::Body,
	extract::{Query, State, WebSocketUpgrade, ws::WebSocket},
	http::StatusCode,
	response::Response,
	routing::any,
};
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tower_http::services::ServeDir;

use std::{
	collections::{HashMap, HashSet},
	time::{Duration, Instant},
};
use thiserror::Error;

// Cookie Clicker runs at 30 fps so there's no reason to go higher...
const FPS: f64 = 30.0;
const BROADCAST_INTERVAL: Duration = Duration::from_millis((1.0 / FPS * 1000.0) as u64);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(6);

type ID = u32; // Don't change this willy-nilly because our update message assumes 32-bit integers.
#[repr(u32)]
enum BinaryType {
	//Add = 1, // Unused since Update is exactly the same...
	Update = 2,
	Remove = 3,
}

#[derive(Serialize, Deserialize, Debug)]
enum JsonToUser {
	#[allow(non_camel_case_types)]
	myid(u32),
}

#[derive(Serialize, Deserialize, Debug)]
enum JsonFromUser {
	#[allow(non_camel_case_types)]
	heartbeat(bool),
}

#[derive(Copy, Clone, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}
#[derive(Error, Debug)]
enum PositionError {
	#[error("Bad message size (message size should be 8 bytes)")]
	BadMessageSize,
	#[error("Position floats are outside of range 0.0 <= float <= 1.0")]
	OutOfBoundsFloats,
}
impl Position {
	fn get(b: &[u8]) -> Result<Self, PositionError> {
		if b.len() == 8 {
			let x = f32::from_le_bytes((&b[0..4]).try_into().unwrap());
			let y = f32::from_le_bytes((&b[4..8]).try_into().unwrap());
			if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) {
				Ok(Position { x, y })
			} else {
				Err(PositionError::OutOfBoundsFloats)
			}
		} else {
			Err(PositionError::BadMessageSize)
		}
	}
}

struct User {
	to: UnboundedSender<MessageToUser>,
	queued_position: Position,
	last_broadcasted_position: Position,
}
struct ServerState {
	latest_id: ID,
	last_broadcast: std::time::Instant,
	users: HashMap<ID, User>,
	queued_updates: HashMap<ID, Position>,
	queued_removes: Vec<ID>,
	queue_timer_task: tokio::task::JoinHandle<()>,
	broadcast_queuer: tokio::sync::mpsc::Sender<()>,
	broadcast_starter: tokio::sync::mpsc::Receiver<()>,
	broadcast_queued: bool,
}
impl Default for ServerState {
	fn default() -> Self {
		let (broadcast_queuer, mut b) = tokio::sync::mpsc::channel(4);
		let (c, broadcast_starter) = tokio::sync::mpsc::channel(4);

		let queue_timer_task = tokio::spawn(async move {
			while let Some(_) = b.recv().await {
				tokio::time::sleep(BROADCAST_INTERVAL).await;
				if let Err(_) = c.blocking_send(()) {
					break;
				}
			}
		});

		ServerState {
			latest_id: 0,
			broadcast_queued: false,
			last_broadcast: Instant::now(),
			users: Default::default(),
			queued_updates: Default::default(),
			queued_removes: Default::default(),
			queue_timer_task: queue_timer_task,
			broadcast_queuer: broadcast_queuer,
			broadcast_starter: broadcast_starter,
		}
	}
}
#[derive(Clone)]
struct AxumState {
	to_server: UnboundedSender<ToServerMessage>,
}
enum ToServerMessage {
	Connect((UnboundedSender<MessageToUser>, tokio::sync::oneshot::Sender<ID>)),
	Remove(ID),
	PositionUpdate((ID, Position)),
}

#[derive(Debug)]
enum MessageToUser {
	Json(JsonToUser),
	Binary(Bytes),
}
impl ServerState {
	fn handle_connect(
		&mut self,
		to_ws: UnboundedSender<MessageToUser>,
		connection_id_tx: tokio::sync::oneshot::Sender<ID>,
	) {
		self.latest_id += 1;
		let id = self.latest_id;
		let _ = connection_id_tx.send(id);
		let len = self.users.len();
		if len > 0 {
			let mut msg = BytesMut::new();
			msg.reserve((2 + (len * 3)) * 4);
			msg.extend_from_slice(&((BinaryType::Update as u32).to_le_bytes()));
			msg.extend_from_slice(&((self.users.len() as u32).to_le_bytes()));
			for (id, user) in &self.users {
				msg.extend_from_slice(&id.to_le_bytes());
				msg.extend_from_slice(&user.last_broadcasted_position.x.to_le_bytes());
				msg.extend_from_slice(&user.last_broadcasted_position.y.to_le_bytes());
			}
			if let Err(_) = to_ws.send(MessageToUser::Binary(msg.freeze())) {
				return;
			}
		}
		assert!(
			self.users
				.insert(
					id,
					User {
						to: to_ws,
						queued_position: Position { x: 0.0, y: 0.0 },
						last_broadcasted_position: Position { x: 0.0, y: 0.0 },
					}
				)
				.is_none()
		);

		println!("join {id} ; connections = {}", self.users.len());
	}
	fn handle_remove(&mut self, id: ID) {
		if self.users.remove(&id).is_some() {
			println!("left {} ; connections = {}", id, self.users.len());
			let _ = self.queued_updates.remove(&id);
			self.queued_removes.push(id);
			self.queue_broadcast();
		}
	}
	fn handle_position_update(&mut self, id: ID, position: Position) {
		let user = self.users.get_mut(&id).unwrap();
		user.queued_position = position;
		if user.last_broadcasted_position == position {
			let _ = self.queued_updates.remove(&id);
		} else {
			let _ = self.queued_updates.insert(id, position);
			self.queue_broadcast();
		}
	}
	fn queue_broadcast(&mut self) {
		let now = Instant::now();
		if now.duration_since(self.last_broadcast) >= BROADCAST_INTERVAL {
			self.broadcast_cursors();
		} else if !self.broadcast_queued {
			self.broadcast_queued = true;
			self.broadcast_queuer.blocking_send(()).unwrap();
		}
	}
	fn broadcast_cursors(&mut self) {
		if self.queued_removes.is_empty() && self.queued_updates.is_empty() {
			// maybe we're here if we queued a broadcast in handle_position_update() but
			// later removed the queued update because they returned to the original position...
			return;
		}

		let all_users: HashSet<ID> = self.users.keys().cloned().collect();
		let updated_users: HashSet<ID> = self.queued_updates.keys().cloned().collect();
		let idle_users: HashSet<ID> = all_users.difference(&updated_users).cloned().collect();

		// println!("{updated_users:?}");
		// println!("{idle_users:?}");

		if !idle_users.is_empty() {
			let msg = self.make_msg_for_idle_users().freeze();
			// println!("msg = {:?}", msg);
			for id in &idle_users {
				let _ = self.users.get(id).unwrap().to.send(MessageToUser::Binary(msg.clone()));
			}
		}

		if !updated_users.is_empty() {
			for id in &updated_users {
				{
					let user = self.users.get_mut(id).unwrap();
					user.last_broadcasted_position = user.queued_position;
				}
				if let Some(msg) = self.make_msg_for_updated_user(*id) {
					let _ = self.users.get(id).unwrap().to.send(MessageToUser::Binary(msg.freeze()));
				}
			}
		}

		self.broadcast_queued = false;
		self.queued_updates.clear();
		self.queued_removes.clear();
	}
	fn make_msg_for_updated_user(&mut self, skip_this_id: ID) -> Option<BytesMut> {
		let mut msg = None;
		// if .len() == 0 then this function wasn't called
		// if .len() == 1 then this user is the only user in the queued updates...
		if self.queued_updates.len() > 1 {
			let msg = msg.get_or_insert(BytesMut::new());
			msg.extend_from_slice(&((BinaryType::Update as u32).to_le_bytes()));
			msg.extend_from_slice(&(((self.queued_updates.len() - 1) as u32).to_le_bytes()));
			for (iter_id, pos) in &self.queued_updates {
				if *iter_id == skip_this_id {
					continue;
				}
				msg.extend_from_slice(&iter_id.to_le_bytes());
				msg.extend_from_slice(&pos.x.to_le_bytes());
				msg.extend_from_slice(&pos.y.to_le_bytes());
			}
		}
		// send ALL removes...
		if !self.queued_removes.is_empty() {
			let msg = msg.get_or_insert_with(BytesMut::new);
			msg.extend_from_slice(&((BinaryType::Remove as u32).to_le_bytes()));
			msg.extend_from_slice(&((self.queued_removes.len() as u32).to_le_bytes()));
			for id in &self.queued_removes {
				msg.extend_from_slice(&id.to_le_bytes());
			}
		}
		msg
	}
	fn make_msg_for_idle_users(&mut self) -> BytesMut {
		let updates_len = if !self.queued_updates.is_empty() {
			2 + (self.queued_updates.len() * 3)
		} else {
			0
		};
		let removes_len = if !self.queued_removes.is_empty() {
			2 + self.queued_removes.len()
		} else {
			0
		};
		let buffer_size = (updates_len + removes_len) * 4;
		assert!(buffer_size > 0); // ?
		let mut msg = BytesMut::new();
		msg.reserve(buffer_size);
		if !self.queued_updates.is_empty() {
			msg.extend_from_slice(&((BinaryType::Update as u32).to_le_bytes()));
			msg.extend_from_slice(&((self.queued_updates.len() as u32).to_le_bytes()));
			for (id, pos) in &self.queued_updates {
				msg.extend_from_slice(&id.to_le_bytes());
				msg.extend_from_slice(&pos.x.to_le_bytes());
				msg.extend_from_slice(&pos.y.to_le_bytes());
			}
		}
		if !self.queued_removes.is_empty() {
			msg.extend_from_slice(&((BinaryType::Remove as u32).to_le_bytes()));
			msg.extend_from_slice(&((self.queued_removes.len() as u32).to_le_bytes()));
			for id in &self.queued_removes {
				msg.extend_from_slice(&id.to_le_bytes());
			}
		}
		msg
	}
}

fn bad_from_query(params: &HashMap<String, String>) -> bool {
	// Sun Aug 18 2024 21:37:36 GMT+0000
	if jiff::Timestamp::now() > jiff::Timestamp::from_second(1_724_017_056).unwrap() {
		// we want "https://cursor-party-0.c.ookie.click/party/rock?from=cc2" and similar...
		// TODO: It's 2024-10-05 and there's still "cc"'s coming in and I'm not sure how, so I give up...
		!params
			.iter()
			.any(|(k, v)| k == "from" && (v == "cc" || v == "cc2" || v == "index"))
	} else {
		false
	}
}

async fn handle_socket(socket: WebSocket, state: AxumState) {
	// axum automagically handles ping/pongs for us

	let (mut ws_s, mut ws_r) = socket.split();
	let (to_me, mut from_server) = tokio::sync::mpsc::unbounded_channel();
	let to_server = state.to_server;

	let (connection_id_tx, connection_id_rx) = tokio::sync::oneshot::channel();

	to_server
		.send(ToServerMessage::Connect((to_me, connection_id_tx)))
		.unwrap();

	let Ok(id) = connection_id_rx.await else {
		return;
	};

	let (send_to_ws_s, mut send_to_ws_r) = tokio::sync::mpsc::unbounded_channel();
	tokio::spawn(async move {
		while let Some(msg) = send_to_ws_r.recv().await {
			let _ = ws_s.send(msg).await;
		}
	});

	let mut last_heartbeat = Instant::now();
	let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);

	loop {
		tokio::select! {
			_ = interval.tick() => {
				if last_heartbeat.elapsed() > CLIENT_TIMEOUT {
					break;
				} else {
					let _ = send_to_ws_s.send(axum::extract::ws::Message::Text(serde_json::to_string(&JsonFromUser::heartbeat(true)).unwrap().into()));
				}
			}
			msg = from_server.recv() => {
				let Some(msg) = msg else {
					break;
				};
				match msg {
					MessageToUser::Json(j) => {
						// TODO: zero-copy...
						let _ = send_to_ws_s.send(axum::extract::ws::Message::Text(serde_json::to_string(&j).unwrap().into()));
					}
					MessageToUser::Binary(b) => {
						let _ = send_to_ws_s.send(axum::extract::ws::Message::Binary(b));
					}
				}
			}
			msg = ws_r.next() => {
				let Some(Ok(msg)) = msg else {
					break;
				};
				// ws::Message::Ping(msg) => {
				// 	self.last_heartbeat = Instant::now();
				// 	ctx.pong(&msg);
				// }
				// ws::Message::Pong(_) => {
				// 	self.last_heartbeat = Instant::now();
				// }
				match msg {
					axum::extract::ws::Message::Text(text) => {
						let Ok(j) = serde_json::from_str::<JsonFromUser>(&text) else {
							break;
						};
						match j {
							JsonFromUser::heartbeat(_) => {
								last_heartbeat = Instant::now();
							}
						}
						// println!("text = '{text}'");
					}
					axum::extract::ws::Message::Binary(b) => {
						let Ok(pos) = Position::get(&b) else {
							break;
						};
						let _ = to_server.send(ToServerMessage::PositionUpdate((id, pos)));
					}
					_ => (),
				}
			}
		}
	}

	let _ = to_server.send(ToServerMessage::Remove(id));
}

async fn handler(
	ws: WebSocketUpgrade,
	Query(params): Query<HashMap<String, String>>,
	State(state): State<AxumState>,
) -> Response {
	if bad_from_query(&params) {
		Response::builder()
			.status(StatusCode::FORBIDDEN)
			.body(Body::empty())
			.unwrap()
	} else {
		ws.on_upgrade(|socket| handle_socket(socket, state))
	}
}

async fn server(mut from_ws: UnboundedReceiver<ToServerMessage>) -> anyhow::Result<()> {
	let mut state = ServerState::default();

	loop {
		tokio::select! {
			_ = state.broadcast_starter.recv() => {
				if state.broadcast_queued {
					state.broadcast_queued = false;
					state.last_broadcast = Instant::now();
					if cfg!(debug_assertions) {
						println!("broadcast {:?}", state.last_broadcast);
					}
					state.broadcast_cursors();
				}
			}
			msg = from_ws.recv() => {
				let Some(msg) = msg else {
					break;
				};
				match msg {
					ToServerMessage::Connect((to_ws, connection_id_tx)) => {
						state.handle_connect(to_ws, connection_id_tx);
					}
					ToServerMessage::Remove(id) => {
						state.handle_remove(id);
					}
					ToServerMessage::PositionUpdate((id, position)) => {
						state.handle_position_update(id, position);
					}
				}
			}
		}
	}

	state.queue_timer_task.abort();

	anyhow::bail!("how are we here...");
}

pub(super) async fn run() -> anyhow::Result<()> {
	let mut tasks = tokio::task::JoinSet::new();

	let (ch_s, ch_r) = tokio::sync::mpsc::unbounded_channel();

	tasks.spawn(server(ch_r));

	let state = AxumState { to_server: ch_s };
	let app = Router::new()
		.fallback_service(ServeDir::new("public/cursor-party-N.c.ookie.click"))
		.route("/party/rock", any(handler))
		.with_state(state);

	if cfg!(debug_assertions) {
		let socket = tokio::net::TcpListener::bind("127.0.0.1:2001").await?;
		tasks.spawn(async move { Ok(axum::serve(socket, app).await?) });
	} else {
		let socket = crate::get_uds("/tmp/c.ookie.click/cursor-party.sock".into()).await?;
		tasks.spawn(async move { Ok(axum::serve(socket, app).await?) });
	}

	while let Some(t) = tasks.join_next().await {
		t??;
	}

	anyhow::bail!("????");
}
