// SPDX-License-Identifier: MIT
// Copyright 2023 Partykit, Inc.
// Copyright 2024 rtldg

// This file is effectively the client-side code of https://github.com/partykit/cursor-party
// but de-React-ed.

const cursorImage =
	"data:image/png;base64,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"
;

document.cursorPartyWs = document.cursorPartyWs || {
	socket: null,

	myid: 0,
	users: new Map(),

	queueMyPositionTimeout: -1,
	updatePartyCountInterval: -1,

	myPositionWindow: [0, 0],
	myPositionDocument: [0, 0],

	windowDimensions: [0, 0],
	scrollDimensions: [0, 0],

	within: "", // "window" or "document"

	divsInitialized: false,
	shouldReconnect: false,
	reconnectTimeout: -1,

	reconnect: function() {
		if (!this.divsInitialized) this.initializeDivs();
		this.close();

		this.shouldReconnect = true;

		if (document.cursorPartyCC == "localhost") {
			this.url = "ws://127.0.0.1:2001/party/rock?from=index";
		} else {
			this.url = `wss://cursor-party-0.c.ookie.click/party/rock?from=${document.cursorPartyCC}`;
		}

		const socket = this.socket = new WebSocket(this.url);
		socket.addEventListener("open", (e) => {
			this.queueMyPosition(); // maybe the socket died, we reconnected, but we are also idle...
		});
		socket.addEventListener("close", this.socketCloseHandler = (e) => {
			this.handleClose();
			if (this.shouldReconnect) {
				this.reconnectTimeout = setTimeout(() => {
					if (this.shouldReconnect) {
						this.reconnect();
					}
				}, 3000);
			}
		});
		socket.addEventListener("message", this.socketMessageHandler = async (e) => {
			if (e.data instanceof Blob) {
				this.handleMessageBinary(await e.data.arrayBuffer());
			} else {
				this.handleMessageJSON(JSON.parse(e.data));
			}
		});
	},
	close: function() {
		this.shouldReconnect = false;
		clearTimeout(this.reconnectTimeout);
		this.reconnectTimeout = -1;
		if (this.socket != null) {
			this.socket.removeEventListener("close", this.socketCloseHandler);
			this.socket.removeEventListener("message", this.socketMessageHandler);
			this.socketCloseHandler({});
			// fucking socket doesn't want to close soon so we just unregister it and fuck off...
			this.socket.close();
			this.socket = null;
		}
	},
	initializeDivs: function() {
		this.useCursorTracking("document");

		//// RIPPED FROM cursor-party/src/cursors.txt
		const cursorsRoot = document.createElement("div");
		document.body.appendChild(cursorsRoot);
		// cursors display is absolute and needs a top-level relative container
		document.documentElement.style.position = "relative";
		document.documentElement.style.minHeight = "100dvh";
		// add a classname
		cursorsRoot.id = "cursorsRoot";
		//// RIPPED FROM cursor-party/src/cursors.txt
		const cursorsSudo = this.cursorsSudo = document.createElement("div");
		cursorsRoot.appendChild(cursorsSudo);
		cursorsSudo.style.position = this.within == "window" ? "fixed" : "absolute";
		cursorsSudo.style.top = 0;
		cursorsSudo.style.left = 0;
		cursorsSudo.style.right = 0;
		cursorsSudo.style.bottom = 0;
		cursorsSudo.style.overflow = "clip";
		cursorsSudo.style.pointerEvents = "none";
		cursorsSudo.id = "cursorsSudo";

		this.updatePartyCountInterval = setInterval(() => {
			document.cursorPartyCount = this.users.size;
		}, 500);
		this.updateScrollDimensionsTimeout = setTimeout(() => {
			this.updateScrollDimensions();
		}, 100);

		this.divsInitialized = true;
	},
	handleClose: function() {
		// this.disableCursorTracking();
		clearTimeout(this.queueMyPositionTimeout);
		this.queueMyPositionTimeout = -1;
		// clearInterval(this.updatePartyCountInterval);
		// this.updatePartyCountInterval = -1;
		this.users.clear();

		this.cursorsSudo.replaceChildren(); // remove all them bitches
	},
	useCursorTracking: function(within) {
		// within can be "window" or "document"
		this.within = within;

		document.cursorPartyMurder = () => {
			// unneeded now...
		};

		window.addEventListener("resize", this.onResizeListener = (e) => this.onResize());
		this.onResize();
		window.addEventListener("mousemove", this.onMouseMoveListener = (e) => this.onMouseMove(e));
		window.addEventListener("touchmove", this.onTouchMoveListener = (e) => this.onMouseMove(e.touches[0]));
		window.addEventListener("scroll", this.onScrollListener = (e) => this.onScroll());
		// window.addEventListener("unload", this.onUnloadListener = (e) => this.onUnload(e));
		// window.addEventListener("onbeforeunload", this.onBeforeUnloadListener = (e) => this.onBeforeUnload(e));
	},
	disableCursorTracking: function() {
		window.removeEventListener("resize", this.onResizeListener);
		window.removeEventListener("mousemove", this.onMouseMoveListener);
		window.removeEventListener("touchmove", this.onTouchMoveListener);
		window.removeEventListener("scroll", this.onScrollListener);
	},
	updateScrollDimensions: function() {
		this.scrollDimensions = [document.documentElement.scrollWidth, document.documentElement.scrollHeight];
		this.transformAllCursors();
	},
	onResize: function() {
		this.updateScrollDimensions();
		this.windowDimensions = [window.innerWidth, window.innerHeight];
		this.transformAllCursors();
	},
	onScroll: function() {
		if (this.within != "document") return;
		this.myPositionDocument = [
			this.myPositionWindow[0] + (window.scrollX || document.documentElement.scrollLeft),
			this.myPositionWindow[1] + (window.scrollY || document.documentElement.scrollTop),
		];
		this.transformAllCursors();
	},
	onMouseMove: function(e) {
		this.myPositionWindow = [e.clientX, e.clientY];
		if (this.within == "document")
			this.myPositionDocument = [e.pageX, e.pageY];
		this.queueMyPosition();
	},
	onUnload: function(e) {
		this.close();
		console.log("onUnload");
	},
	onUnload: function(e) {
		this.close();
		console.log("onBeforeUnload");
	},
	transformAllCursors: function() {
		// - transform all cursors from users
		this.users.forEach((v, k, m) => this.updateUserPositionInternal(v));
	},
	createCursor: function(id, untransformed_pos) {
		const div = document.createElement("div");
		const u = [untransformed_pos, null, div];
		this.updateUserPositionInternal(u);
		this.users.set(id, u);
		document.getElementById("cursorsSudo").appendChild(div);
		div.id = `cursor-user-${id}`;
		div.style.opacity = 1.0;
		div.style.zIndex = 1001;
		div.style.position = "absolute";
		// div.style.transform = `translate(${transformed_pos[0] - 10}px, ${transformed_pos[1] - 10}px)`;
		// div.style.left = `${pos[0] * window.innerWidth - 10}px`;
		// div.style.top = `${pos[1] * window.innerHeight - 10}px`;
		const img = document.createElement("img");
		img.style.transform = "scale(.5)";
		img.src = cursorImage;
		div.appendChild(img);
	},
	transformCursor: function(cur) {
		const bounds = this.within == "window" ? this.windowDimensions : this.scrollDimensions;
		return [cur[0] * bounds[0], cur[1] * bounds[1]];
	},
	updateUserPositionInternal: function(u) {
		u[1] = this.transformCursor(u[0]);
		u[2].style.transform = `translate(${u[1][0] - 10}px, ${u[1][1] - 10}px)`;
		// u[1].style.left = `${pos[0] * window.innerWidth - 10}px`;
		// u[1].style.top = `${pos[1] * window.innerHeight - 10}px`;
	},
	updateUserPosition: function(id, untransformed_pos) {
		if (id == this.myid)
			return;
		let u = this.users.get(id);
		if (u == undefined) {
			this.createCursor(id, untransformed_pos);
		} else {
			u[0] = untransformed_pos;
			this.updateUserPositionInternal(u);
		}
	},
	queueMyPosition: function() {
		if (this.socket != null && this.socket.readyState == 1 && this.queueMyPositionTimeout == -1) {
			this.queueMyPositionTimeout = setTimeout(() => {
				if (this.socket == null) return;
				this.queueMyPositionTimeout = -1;
				const bounds = this.within == "window" ? this.windowDimensions : this.scrollDimensions;
				const cur = this.within == "window" ? this.myPositionWindow : this.myPositionDocument;
				const buffer = new ArrayBuffer(8);
				const view = new DataView(buffer);
				view.setFloat32(0, cur[0] / bounds[0], true); // little-endian float...
				view.setFloat32(4, cur[1] / bounds[1], true); // little-endian float...
				this.socket.send(buffer);
			}, 25);
		}
	},
	removeUser: function(id) {
		this.users.delete(id);
		document.getElementById("cursor-user-" + id).remove();
	},
	handleMessageBinary: function(buffer) {
		let pos = 0;
		const view = new DataView(buffer);
		for (const _c = view.byteLength / 4; pos < _c;) {
			// we read all these values as little-endian
			const type = view.getUint32(pos++ * 4, true);
			const count = view.getUint32(pos++ * 4, true);
			if (!count) continue;
			if (type == 1 || type == 2) {
				// add || presence
				for (let i = 0; i < count; i++) {
					const id = view.getUint32(pos++ * 4, true);
					const x = view.getFloat32(pos++ * 4, true);
					const y = view.getFloat32(pos++ * 4, true);
					this.updateUserPosition(id, [x, y]);
				}
			} else if (type == 3) {
				for (let i = 0; i < count; i++) {
					this.removeUser(view.getUint32(pos++ * 4, true));
				}
			}
		}
	},
	handleMessageJSON: function(msg) {
		if (msg.myid) {
			this.myid = msg.myid;
		} else if (msg.heartbeat) {
			this.socket.send(`{"heartbeat":true}`);
		}
	},
};
document.cursorPartyWs.reconnect();
