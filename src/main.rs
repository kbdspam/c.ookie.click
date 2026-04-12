// SPDX-License-Identifier:
// Copyright

#![forbid(unsafe_code)]

use actix::prelude::*;
use actix_web::{
	web::{self, Bytes, BytesMut},
	HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};

use std::{
	collections::{HashMap, HashSet},
	time::{Duration, Instant},
};
use thiserror::Error;

// Cookie Clicker runs at 30 fps so there's no reason to go higher...
const FPS: f64 = 30.0;
const BROADCAST_INTERVAL: Duration = Duration::from_millis((1.0 / FPS * 1000.0) as u64);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
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
struct Session {
	id: ID,
	addr: Addr<State>,
	last_heartbeat: Instant,
}
impl Session {
	fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
				ctx.stop();
			} else {
				ctx.text(serde_json::to_string(&JsonFromUser::heartbeat(true)).unwrap().as_str());
			}
		});
	}
}
impl Actor for Session {
	type Context = ws::WebsocketContext<Self>;
	fn started(&mut self, ctx: &mut Self::Context) {
		self.heartbeat(ctx);
		self.addr
			.send(Connect {
				addr: ctx.address().recipient(),
			})
			.into_actor(self)
			.then(|res, act, ctx| {
				match res {
					Ok(res) => act.id = res,
					_ => ctx.stop(),
				}
				fut::ready(())
			})
			.wait(ctx);
	}
	fn stopping(&mut self, _: &mut Self::Context) -> Running {
		self.addr.do_send(Remove(self.id));
		Running::Stop
	}
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
struct User {
	addr: Recipient<MessageToUser>,
	queued_position: Position,
	last_broadcasted_position: Position,
}
struct State {
	latest_id: u32,
	broadcast_queued: bool,
	last_broadcast: std::time::Instant,
	users: HashMap<ID, User>,
	queued_updates: HashMap<ID, Position>,
	queued_removes: Vec<ID>,
}
impl Default for State {
	fn default() -> Self {
		State {
			latest_id: 0,
			broadcast_queued: false,
			last_broadcast: std::time::Instant::now(),
			users: Default::default(),
			queued_updates: Default::default(),
			queued_removes: Default::default(),
		}
	}
}
impl Actor for State {
	type Context = Context<Self>;
}
#[derive(Message, Debug)]
#[rtype(result = "()")]
enum MessageToUser {
	Json(JsonToUser),
	Binary(Bytes),
}

#[derive(Message)]
#[rtype(ID)]
struct Connect {
	addr: Recipient<MessageToUser>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct PositionUpdate(ID, Position);
impl Handler<PositionUpdate> for State {
	type Result = ();
	fn handle(&mut self, msg: PositionUpdate, ctx: &mut Self::Context) -> Self::Result {
		let user = self.users.get_mut(&msg.0).unwrap();
		user.queued_position = msg.1;
		if user.last_broadcasted_position == msg.1 {
			let _ = self.queued_updates.remove(&msg.0);
		} else {
			let _ = self.queued_updates.insert(msg.0, msg.1);
			self.queue_broadcast(ctx);
		}
	}
}
#[derive(Message)]
#[rtype(result = "()")]
struct Remove(ID);
impl Handler<Remove> for State {
	type Result = ();
	fn handle(&mut self, msg: Remove, ctx: &mut Self::Context) -> Self::Result {
		if self.users.remove(&msg.0).is_some() {
			println!("left {} ; connections = {}", msg.0, self.users.len());
			let _ = self.queued_updates.remove(&msg.0);
			self.queued_removes.push(msg.0);
			self.queue_broadcast(ctx);
		}
	}
}
impl Handler<Connect> for State {
	type Result = ID;
	fn handle(&mut self, connect_msg: Connect, _: &mut Context<Self>) -> Self::Result {
		self.latest_id += 1;
		let id = self.latest_id;
		connect_msg.addr.do_send(MessageToUser::Json(JsonToUser::myid(id)));
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
			connect_msg.addr.do_send(MessageToUser::Binary(msg.freeze()));
		}
		assert!(self
			.users
			.insert(
				id,
				User {
					addr: connect_msg.addr,
					queued_position: Position { x: 0.0, y: 0.0 },
					last_broadcasted_position: Position { x: 0.0, y: 0.0 },
				}
			)
			.is_none());

		println!("join {id} ; connections = {}", self.users.len());

		id
	}
}
impl State {
	fn queue_broadcast(&mut self, ctx: &mut <State as actix::Actor>::Context) {
		let now = std::time::Instant::now();
		if now.duration_since(self.last_broadcast) >= BROADCAST_INTERVAL {
			self.broadcast_cursors();
		} else if !self.broadcast_queued {
			self.broadcast_queued = true;
			ctx.run_later(BROADCAST_INTERVAL, |act, _ctx| {
				act.last_broadcast = std::time::Instant::now();
				if cfg!(debug_assertions) {
					println!("broadcast {:?}", std::time::Instant::now());
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

fn bad_from_query(req: &HttpRequest) -> bool {
	// Sun Aug 18 2024 21:37:36 GMT+0000
	if time::OffsetDateTime::now_utc().unix_timestamp() > 1724017056 {
		// we want "https://cursor-party-0.c.ookie.click/party/rock?from=cc2" and similar...
		// TODO: It's 2024-10-05 and there's still "cc"'s coming in and I'm not sure how so I give up...
		!req.full_url()
			.query_pairs()
			.any(|(k, v)| k == "from" && (v == "cc" || v == "cc2" || v == "index"))
	} else {
		false
	}
}

async fn handle_websocket(
	req: HttpRequest,
	stream: web::Payload,
	state: web::Data<Addr<State>>,
) -> Result<HttpResponse, actix_web::Error> {
	if bad_from_query(&req) {
		return Ok(HttpResponse::Forbidden().finish());
	}

	ws::start(
		Session {
			id: 0,
			addr: state.get_ref().clone(),
			last_heartbeat: Instant::now(),
		},
		&req,
		stream,
	)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	// let state: SharedState = Default::default();
	let state = State::default().start();
	HttpServer::new(move || {
		actix_web::App::new()
			.app_data(web::Data::new(state.clone()))
			.route("/party/rock", web::get().to(handle_websocket))
			// >If the mount path is set as the root path /, services registered after this one will be inaccessible. Register more specific handlers and services first.
			.service(
				actix_files::Files::new("/", "./public/cursor-party-N.c.ookie.click/")
					.prefer_utf8(true)
					.index_file("index.html"),
			)
	})
	// TODO: .bind_uds() on Linux for a unix socket...
	.bind(("127.0.0.1", 2001))?
	.run()
	.await
}
