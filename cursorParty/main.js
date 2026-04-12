
Game.registerMod("cursorParty", {
	init: function() {
		if (!(this.goodInit = ["3268177089", "3061304069"].includes(App.mods['cursorParty'].workshop)))
			return;

		// If we don't have mod data yet (first run) then the load() function won't run.
		// Hook some function to simplify ourself.
		if (!("loadModData_original" in Game)) {
			Game.loadModData_post_functions = {};
			Game.loadModData_original = Game.loadModData;
			Game.loadModData = function() {
				Game.loadModData_original();
				for (let key in Game.loadModData_post_functions) {
					Game.loadModData_post_functions[key]();
				}
			};
		}
		Game.loadModData_post_functions["cursorParty"] = () => {
			delete Game.loadModData_post_functions["cursorParty"];
			if (!this.load_called) this.load("on");
		};
		/*
		// Deleting mod data doesn't work if the plugin is still loaded/enabled.
		// Delete mod data -> save & quit -> mod saves data again -> oops...
		Game.original_deleteModData2 = Game.deleteModData;
		Game.deleteModData = (id) => {
			Game.original_deleteModData2(id);
			if (id == "cursorParty") {}//this.initDatas(null);
		};
		Game.original_deleteAllModData2 = Game.deleteAllModData;
		Game.deleteAllModData = () => {
			Game.original_deleteAllModData2();
			//this.initDatas(null);
		};
		*/

		l('versionNumber').insertAdjacentHTML('beforeend','<a style="font-size:10px;margin-left:10px;" class="smallFancyButton" id="cursorPartyButton">Cursor Party<br>OFF</a>');
		AddEvent(l('cursorPartyButton'),'click',()=>{
			if (this.blockToggle) {
				PlaySound('snd/clickOff2.mp3');
				return;
			}
			if (this.active) {
				this.stopTheCursors();
				PlaySound('snd/clickOff2.mp3');
			} else {
				this.startTheCursors();
				PlaySound('snd/clickOn2.mp3');
			}
			setInterval(()=>{this.blockToggle = false;}, 1*1000);
		});
	},
	injectScript: function(subdomain) {
		document.cursorPartyCC = "cc2";
		let script = document.createElement("script");
		//script.src = "http://localhost:1999/cursors.js";
		script.src = `https://cursor-party-${subdomain}.c.ookie.click/cursors2.js`;
		document.body.appendChild(script);

		script.onload = () => {
			setInterval(() => this.updateDisplay(), 2500);
			this.blockToggle = false;
			l("cursorPartyButton").innerHTML = "Cursor Party<br>ON";
		};
		script.onerror = () => {
			console.log(`failed to load cursors.js from ${subdomain}...`);
			if (subdomain < 3) {
				this.injectScript(subdomain + 1);
			} else {
				setTimeout(() => {
					this.injectScript(0);
				}, 3 * 60 * 1000);
			}
		};
	},
	startTheCursors: function() {
		if ("cursorPartyWs" in document) {
			// The cursors.js script sets `document.cursorPartyWs` to the socket...
			document.cursorPartyWs.reconnect();
			l("cursorPartyButton").innerHTML = "Cursor Party<br>ON";
		} else {
			this.blockToggle = true;
			l("cursorPartyButton").innerHTML = "Cursor Party<br>loading...";
			this.injectScript(0);
		}
		document.cursorPartyCount = 0;
		this.active = true;
	},
	stopTheCursors: function() {
		if ("cursorPartyWs" in document) {
			document.cursorPartyMurder(); // clears cursors...
			document.cursorPartyWs.close();
		}
		l("cursorPartyButton").innerHTML = "Cursor Party<br>OFF";
		this.active = false;
	},
	updateDisplay: function() {
		if (!this.active) return;
		if (!document.cursorPartyCount) return;
		l("cursorPartyButton").innerHTML =
			`Cursor Party<br>Connected: ${document.cursorPartyCount+1}`;
	},
	save:function() {
		return this.active ? "on" : "off";
	},
	load:function(str) {
		if (!this.goodInit) return;
		this.load_called = true;
		if (this.active) return;
		if (str != "off") this.startTheCursors();
	},
});
