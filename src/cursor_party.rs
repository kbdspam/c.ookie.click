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

enum AppMessage {
	Connect(ID),
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
	to: UnboundedSender<ToWsMessage>,
	queued_position: Position,
	last_broadcasted_position: Position,
}
struct ServerState {
	latest_id: ID,
	broadcast_queued: bool,
	last_broadcast: std::time::Instant,
	users: HashMap<ID, User>,
	queued_updates: HashMap<ID, Position>,
	queued_removes: Vec<ID>,
}
impl Default for ServerState {
	fn default() -> Self {
		ServerState {
			latest_id: 0,
			broadcast_queued: false,
			last_broadcast: Instant::now(),
			users: Default::default(),
			queued_updates: Default::default(),
			queued_removes: Default::default(),
		}
	}
}
struct AxumState {
	to_server: UnboundedSender<ToServerMessage>,
}
enum ToServerMessage {
	Connect(UnboundedSender<ToWsMessage>),
	Remove(ID),
	PositionUpdate((ID, Position)),
}
enum ToWsMessage {
	MessageToUser(MessageToUser),
	ConnectResp(ID),
}

impl Handler<MessageToUser> for Session {
	type Result = ();
	fn handle(&mut self, msg: MessageToUser, ctx: &mut Self::Context) -> Self::Result {
		// println!("sending {:?} to {}", msg, self.id);
		match msg {
			MessageToUser::Json(j) => {
				// TODO: zero-copy...
				ctx.text(serde_json::to_string(&j).unwrap().as_str());
			}
			MessageToUser::Binary(b) => {
				ctx.binary(b);
			}
		}
	}
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		let Ok(msg) = msg else {
			ctx.stop();
			return;
		};
		match msg {
			// ws::Message::Ping(msg) => {
			// 	self.last_heartbeat = Instant::now();
			// 	ctx.pong(&msg);
			// }
			// ws::Message::Pong(_) => {
			// 	self.last_heartbeat = Instant::now();
			// }
			ws::Message::Text(text) => {
				let Ok(j) = serde_json::from_str::<JsonFromUser>(&text) else {
					ctx.stop();
					return;
				};
				match j {
					JsonFromUser::heartbeat(_) => {
						self.last_heartbeat = Instant::now();
					}
				}
				// println!("text = '{text}'");
			}
			ws::Message::Binary(b) => {
				let Ok(pos) = Position::get(&b) else {
					ctx.stop();
					return;
				};
				self.addr.do_send(PositionUpdate(self.id, pos));
			}
			_ => (),
		}
	}
}
#[derive(Debug)]
enum MessageToUser {
	Json(JsonToUser),
	Binary(Bytes),
}
impl ServerState {
	fn handle_connect(&mut self, to_ws: UnboundedSender<ToWsMessage>) {
		self.latest_id += 1;
		let id = self.latest_id;
		to_ws.send(ToWsMessage::ConnectResp(id));
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
			to_ws.send(ToWsMessage::MessageToUser(MessageToUser::Binary(msg.freeze())));
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
			ctx.run_later(BROADCAST_INTERVAL, |act, _ctx| {
				act.last_broadcast = Instant::now();
				if cfg!(debug_assertions) {
					println!("broadcast {:?}", act.last_broadcast);
				}
				act.broadcast_cursors();
			});
		}
	}
	fn broadcast_cursors(&mut self) {
		if self.queued_removes.is_empty() && self.queued_updates.is_empty() {
			// maybe we're here if we queued a broadcast in Handler<PositionUpdate>::handle() but
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
				self.users
					.get(id)
					.unwrap()
					.addr
					.do_send(MessageToUser::Binary(msg.clone()));
			}
		}

		if !updated_users.is_empty() {
			for id in &updated_users {
				{
					let user = self.users.get_mut(id).unwrap();
					user.last_broadcasted_position = user.queued_position;
				}
				if let Some(msg) = self.make_msg_for_updated_user(*id) {
					self.users
						.get(id)
						.unwrap()
						.addr
						.do_send(MessageToUser::Binary(msg.freeze()));
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
		!params.any(|(k, v)| k == "from" && (v == "cc" || v == "cc2" || v == "index"))
	} else {
		false
	}
}

async fn handle_socket(socket: WebSocket, state: AxumState) {
	// axum automagically handles ping/pongs for us

	let (mut ws_s, mut ws_r) = socket.split();
	let (ch_s, mut ch_r) = tokio::sync::mpsc::unbounded_channel();
	let to_server = state.to_server;
	to_server.send(ToServerMessage::Connect(ch_s)).unwrap();

	let ToWsMessage::ConnectResp(id) = ch_r.recv().await.unwrap() else {
		return;
	};

	tokio::spawn(async move {
		while let Some(msg) = ch_r.recv().await {
			let _ = ws_s.send(msg).await;
		}
	});

	let mut last_heartbeat = Instant::now();
	let mut interval = tokio::time::interval(Duration::from_secs(1));

	loop {
		tokio::select! {
			_ = interval.tick() => {
				if last_heartbeat.elapsed() > CLIENT_TIMEOUT {
					// TODO: kill
				} else {
					//ctx.text(serde_json::to_string(&JsonFromUser::heartbeat(true)).unwrap().as_str());
				}
			}
		}
	}

	// TODO:
	to_server.send(ToServerMessage::Remove(id));

	/*
	tasks.spawn(async move {
		while let Ok(_) = ws_s.send(axum::extract::ws::Message::Text(":3".into())).await {
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
	*/
}

pub(super) async fn handler(
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

	while let Some(msg_from_ws) = from_ws.recv().await {
		match msg_from_ws {
			ToServerMessage::Connect(to_ws) => {
				state.handle_connect(to_ws);
			}
			ToServerMessage::Remove(id) => {
				state.handle_remove(id);
			}
			ToServerMessage::PositionUpdate((id, position)) => {
				state.handle_position_update(id, position);
			}
		}
	}

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

	#[cfg(debug_assertions)]
	tasks.spawn(axum::serve(tokio::net::TcpListener::bind("127.0.0.1:8081").await?, app));
	#[cfg(not(debug_assertions))]
	tasks.spawn(axum::serve(
		crate::get_uds("/tmp/c.ookie.click/cursor-party.sock".into()).await?,
		app,
	));

	while let Some(t) = tasks.join_next().await {
		t??;
	}

	anyhow::bail!("????");
}
