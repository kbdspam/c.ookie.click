
// this was intentionally written sloppily teehee

// init() called first. if a save exists then load()...

Game.registerMod("ookieLeaderboard",{
	dev: 0,

	save: function() {
		return JSON.stringify(this.settings);
	},
	load: function(str) {
		this.initDatas(str);
		setTimeout(()=>document.ookieLeaderboard.leaderboard_updateme(),2*1000);//bleh
		this.leaderboard_query();
	},
	initDatas: function(str) {
		//if (this.dev) str = '{"cookiedev":"none","cookiereal":"none"}';
		this.settings = JSON.parse(str||'{"cookiedev":"none","cookiereal":"none"}');
		this.cookie = this.dev ? this.settings.cookiedev : this.settings.cookiereal;
		if (l("leaderboardTabPage")) l("leaderboardTabPage").remove();
		if (this.cookie == "none") {
			l("leaderboardTabBar").innerHTML = `
				<a style="font-size:12px;" class="smallFancyButton" onclick="ookieLeaderboard.registerButton()">register</a>
			`;
		}
	},


	leaderboard_updateme: function() {
		if (this.updatemeInterval) clearInterval(this.updatemeInterval);
		this.updatemeInterval = setInterval(()=>document.ookieLeaderboard.leaderboard_updateme(), this.updateS*1000); // dumb, I know
		if (this.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/updateme", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.cookie,
				"X-My-Update-Data": Game.cookiesEarned+'|'+Game.cookiesPsRaw,
			},
		}).then(response => {});
	},
	leaderboard_leave: function(id) {
		if (this.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/leave", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.cookie,
				"X-My-Leaderboard-ID": id.toString(),
			},
		}).then(response => {
			if (!response.ok) throw Error(response.statusText);
			this.leaderboard_query();
		}).catch(err => {
			Game.Notify('failed to join leaderboard','The server might be down...',0,5);
		});
	},
	leaderboard_join: function(cookie) {
		if (this.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/join", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.cookie,
				"X-My-Leaderboard-Cookie": cookie,
			},
		}).then(response => {
			if (!response.ok) throw Error(response.statusText);
			this.leaderboard_query();
		}).catch(err => {
			Game.Notify('failed to join leaderboard','The server might be down...',0,5);
		});
	},
	leaderboard_create: function(name) {
		if (this.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/create", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.cookie,
				"X-My-New-Leaderboard-Name": name,
			},
		}).then(response => {
			this.waitingForRegister = false;
			if (!response.ok) throw Error(response.statusText);
			return response.text();
		}).then(cookie => {
			this.leaderboard_query();
		}).catch(err => {
			Game.Notify('failed to create leaderboard','The server might be down...',0,5);
		});
	},
	leaderboard_kick: function(board,id) {
		if (this.cookie == "none") return;
		if (this.you == id) return; //?
		fetch(this.baseURL+"/leaderboard/kick", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.cookie,
				"X-My-Leaderboard-ID": board.toString(),
				"X-My-Enemy-ID": id.toString(),
			},
		}).then(response => {
			if (!response.ok) throw Error(response.statusText);
			this.leaderboard_query();
			Game.Notify('kicked!','',0,5);
		}).catch(err => {
			Game.Notify('failed to kick that person','The server might be down or maybe they\'re not in the group anymore...',0,5);
		});
	},
	leaderboard_cycleboardcookie: function(id) {
		if (this.cookie == "none") return;
		if (this.rateLimit(5, "lastcycle")) return;
		fetch(this.baseURL+"/leaderboard/cycleboardcookie", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.cookie,
				"X-My-Leaderboard-ID": id.toString(),
			},
		}).then(response => {
			if (!response.ok) throw Error(response.statusText);
			this.leaderboard_query();
			Game.Notify('leaderboard invite code changed!','',0,5);
		}).catch(err => {
			Game.Notify('failed to change invite code','The server might be down...',0,5);
		});
	},
	leaderboard_register: function(name) {
		if (this.cookie != "none") return;
		if (this.waitingForRegister) return;
		this.waitingForRegister = true;
		fetch(this.baseURL+"/leaderboard/register", {
			method: "POST",
			headers: {
				"X-My-New-Leaderboard-Name": name,
			},
		}).then(response => {
			this.waitingForRegister = false;
			if (!response.ok) throw Error(response.statusText);
			return response.text();
		}).then(cookie => {
			if (this.dev) {
				if (this.settings.cookiedev == "none")
					this.settings.cookiedev = cookie;
			} else {
				this.settings.cookiereal = cookie;
			}
			this.cookie = cookie;
			this.leaderboard_updateme();
			setTimeout(()=>{this.leaderboard_query();}, 1*1000);
			Game.Notify("Registered!",'',0,5);
		}).catch(err => {
			// TODO more stuff here...
			Game.Notify("Failed to register :/",'The server might be down...',0,5);
		});
	},
	leaderboard_query: function() {
		if (this.queryInterval) clearInterval(this.queryInterval);
		this.queryInterval = setInterval(()=>document.ookieLeaderboard.leaderboard_query(), this.queryS*1000); // dumb, I know
		const MOD = this;
		if (this.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/query", {
			headers: {
				"X-My-Cookie": this.cookie,
			},
		}).then(response => {
			if (!response.ok) throw Error(response.statusText);
			return response.json();
		}).then(json => {
			this.queriedOnce = true;
			this.you = json.you;
			this.can_mod = +json.can_mod;
			l("leaderboardProductMod").style.display = this.can_mod ? "" : "none";
			//console.log(json);
			// setup tab bar from json.boardinfo [[boardid, boardname],]
			let newTabBar = "";
			let boards = {};
			let foundCurrent = false;
			for (let b of json.boardinfo) {
				boards[b[0]] = {values:[],unsafe_name:b[1],cookie:b[2],myrank:"9001"};
				if (b[0] == this.tabOpenTo) foundCurrent = true;
			}
			// parse json.boardvalues to fill out MOD.boardvalues[]
			let lastboard = null, rank = 0;
			for (let v of json.boardvalues) {
				// SELECT j.board, c.name, c.total_cookies, c.cookies_per_second, c.id
				const boardid = v.splice(0,1)[0];
				if (boardid != lastboard) [lastboard,rank] = [boardid,0];
				++rank;
				boards[boardid].values.push(v);
				if (+v[3] == this.you) boards[boardid].myrank = rank;
			}
			for (const board in boards) {
				newTabBar += `<div class="leaderboardTab subButton" id="leaderboardTab${board}">#${boards[board].myrank} / ${boards[board].values.length}<br>${this.escapeHTML(boards[board].unsafe_name)}</div>`;
			}
			if (newTabBar == "") {
				newTabBar = `
					<div style="padding: 10px 10px 10px 10px;">try joining or creating a leaderboard with the button to the bottom right</div>
				`;
			}
			l("leaderboardTabBar").innerHTML = newTabBar;
			document.querySelectorAll(".leaderboardTab").forEach((tab)=>AddEvent(tab,'click',(e)=>MOD._leaderboardTabClick(e)));
			this.boards = boards;
			if (foundCurrent) this.viewLeaderboardPage(this.tabOpenTo, l('leaderboardTabPage').scrollTop);
			else if (l("leaderboardTabPage")) l("leaderboardTabPage").remove(); // maybe the client got kicked
		}).catch(()=>{
			if (l("leaderboardTabBar").innerText == "loading...")
				l("leaderboardTabBar").innerText += " (server might be down)";
		});
	},


	joinGlobalPrompt: function() {
		if (this.cookie == "none") return this.registerButton();
		if (!this.queriedOnce) return;
		if (1 in this.boards) return;
		if (Object.keys(this.boards).length >= 5) {
			Game.Notify("You can't join/create any more leaderboards!",'',0,5);
			PlaySound('snd/clickOff2.mp3');
			return;
		}
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt('<id LeaderboardJoinGlobalXXXX><h3>Join the Global🌎 leaderboard?</h3><div class="block" style="text-align:center;">If you have a bad or unverified name then it will be blurred for other players until it has been checked or changed.<br></div>', [
			["join", `
				ookieLeaderboard.leaderboard_join("gJKI0tXJEHSBGrEYRVv5pe6rNQcY1RxT");
				Game.ClosePrompt();
			`],
			loc("Cancel"),
		]);
	},
	joinCreatePrompt: function() {
		if (this.cookie == "none") return this.registerButton();
		if (!this.queriedOnce) return;
		if (Object.keys(this.boards).length >= 5) {
			Game.Notify("You can't join/create any more leaderboards!",'',0,5);
			PlaySound('snd/clickOff2.mp3');
			return;
		}
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt('<id LeaderboardJoinOrCreateXXXX><h3>Join or Create a Leaderboard</h3><div class="block" style="text-align:center;">insert a name or a leaderboard-cookie</div><div class="block"><input type="text" style="text-align:center;width:100%;" id="leaderboardJoinOrCreate" value=""/></div>', [
			["join", `
				const s = l('leaderboardJoinOrCreate').value.trim();
				if (s.length == 32 && s.match(/^[0-9a-zA-Z]+$/)) {
					ookieLeaderboard.leaderboard_join(s);
					Game.ClosePrompt();
				}
			`],
			["create", `
				const s = l('leaderboardJoinOrCreate').value.trim();
				if (ookieLeaderboard.isOkayName(s)) {
					ookieLeaderboard.leaderboard_create(s);
					Game.ClosePrompt();
				}
			`],
			loc("Cancel"),
		]);
		l('leaderboardJoinOrCreate').focus();
		l('leaderboardJoinOrCreate').select();
	},
	_leaderboardTabClick: function(e) {
		const item = e.target.id.split("leaderboardTab")[1]; // this is so stupid but it's so easy
		if (this.tabOpenTo == +item) {
			this.tabOpenTo = null;
			PlaySound('snd/clickOff2.mp3');
			l("leaderboardTab"+item).classList.remove("leaderboardTabSelected");
			if (l("leaderboardTabPage")) l("leaderboardTabPage").remove();
		} else {
			PlaySound('snd/clickOn2.mp3');
			this.viewLeaderboardPage(+item);
		}
	},
	registerButton: function() {
		if (this.cookie != "none") return;
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt('<id LeaderboardRegisterAAAA><h3>Register on c.ookie.click/er/</h3><div class="block" style="text-align:center;">Enter a name you want to use on leaderboards. This can\'t be changed afterwards. Don\'t choose something racist, sexist, offensive, etc please.</div><div class="block"><input type="text" style="text-align:center;width:100%;" id="leaderboardRegisterPrompt" value=""/></div>', [
			["register", `
				const s = l('leaderboardRegisterPrompt').value.trim();
				if (ookieLeaderboard.isOkayName(s)) {
					ookieLeaderboard.leaderboard_register(s);
					Game.ClosePrompt();
				}
			`],
			loc("Cancel"),
		]);
		l('leaderboardRegisterPrompt').focus();
		l('leaderboardRegisterPrompt').select();
	},
	leaveButton: function() {
		if (this.tabOpenTo == null) return;//?
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt('<id LeaderboardLeaveAAAAA><h3>Are you sure you want to leave?</h3><div class="block" style="text-align:center;">if you have double-checked that you\'re leaving the correct leaderboard then type in "sayonara" (without the quotes) and hit leave<br>(also if you\'re the owner then leaving will delete the leaderboard)</div><div class="block"><input type="text" style="text-align:center;width:100%;" id="leaderboardLeavePrompt" value=""/></div>', [
			["leave", `
				const s = l('leaderboardLeavePrompt').value.trim();
				if (s == "sayonara" || s == "さよなら" || s.startsWith("じゃ")) {
					ookieLeaderboard.leaderboard_leave(ookieLeaderboard.tabOpenTo);
					Game.ClosePrompt();
					Game.Notify("goodbye :(",'',0,5);
				}
			`],
			loc("Cancel"),
		]);
		l('leaderboardLeavePrompt').focus();
		l('leaderboardLeavePrompt').select();
	},
	kickButton: function(butt,board,id) {
		if (this.tabOpenTo == null) return;//?
		const name = butt.parentElement.parentElement.children[0].innerHTML;
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt(`<id LeaderboardKickerAA><h3>Kick ${name}?</h3><div class="block" style="text-align:center;">type anything in the box below and hit 'kick' to confirm<br>(kicking someone also changes the invite code)</div><div class="block"><input type="text" style="text-align:center;width:100%;" id="leaderboardKickPrompt" value=""/></div>`, [
			["kick", `
				if (l('leaderboardKickPrompt').value.length > 0) {
					ookieLeaderboard.leaderboard_kick(${board},${id});
					Game.ClosePrompt();
				}
			`],
			loc("Cancel"),
		]);
		l('leaderboardKickPrompt').focus();
		l('leaderboardKickPrompt').select();
	},
	cycleInviteButton: function() {
		if (this.tabOpenTo == null) return;
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt(`<id LeaderboardCycleCodeAAAA><h3>Are you sure you want to change the invite code?</h3><div class="block" style="text-align:center;">bleh</div>`, [
			[loc("Yes"), `
				ookieLeaderboard.leaderboard_cycleboardcookie(${this.tabOpenTo});
				Game.ClosePrompt();
			`],
			loc("No"),
		]);
	},
	viewLeaderboardPage: function(board, scrollTop) {
		this.tabOpenTo = board;
		if (l("leaderboardTabPage")) l("leaderboardTabPage").remove();
		for (let e of document.querySelectorAll(".leaderboardTab")) {
			if (e.id == "leaderboardTab"+board) {
				e.classList.add("leaderboardTabSelected");
			} else {
				e.classList.remove("leaderboardTabSelected");
			}
		}
		/*let bcookie = '';
		for (const e of this.boardinfo) {
			if (e[0] == board) bcookie = e[2];
		}*/
		const bcookie = this.boards[board].cookie;
		let page = `
			<div id="leaderboardTabPage">
				<table>
				<thead>
				<tr id="leaderboardTabPageTable">
				<th style="width:28%">Name</th>
				<th style="width:36%">${loc("Raw cookies per second:")}</th>
				<th style="width:40%">${loc("Cookies baked (this ascension):")}</th>
				</tr>
				</thead>
				<tbody id="leaderboardTabPageTBody">
		`;
		for (const v of this.boards[board].values) {
			// c.name, c.total_cookies, c.cookies_per_second, c.id, c.okay_name

			// The "global" leaderboard is board-ID 1. We only blur unchecked or :( names on this board.
			const isyou = (+v[3]==this.you);
			const blur = (!isyou && board==1 && !v[4]) ? ' class="blur"' : "";
			const style = isyou ? ' style="outline: rgba(255,255,255,.3) solid 2px"' : '';
			// lol. invisible button for uniform row size... just fix the padding lol TODO
			const kickb = (bcookie=='')?'' : ((+v[3]==this.you)?`<td><a class="smallFancyButton" style="visibility: hidden;">kick</a></td>`:`<td><a class="smallFancyButton" onclick="document.ookieLeaderboard.kickButton(this,${board},${v[3]})">kick</a></td>`);
			page += `
				<tr${style}>
				<td${blur}>${this.escapeHTML(v[0])}</td>
				<td>${Beautify(v[1])}</td>
				<td>${Beautify(v[2])}</td>
				${kickb}
				</tr>
			`;
		}
		l('leaderboardTabBar').insertAdjacentHTML('afterend',
			page+`
				</tbody>
				</table>
				<a class="smallFancyButton" id="leaderboardLeave" onclick="document.ookieLeaderboard.leaveButton()">leave</a>
			`+(bcookie==''?"":`
				<a class="smallFancyButton" id="leaderboardTabGetCode" onclick="navigator.clipboard.writeText('${bcookie}').then(()=>Game.Notify('copied leaderboard invite code to clipboard','',0,5));">copy invite code</a>
				<!-- Hello there -->
				<a class="smallFancyButton" id="leaderboardTabCycleCode" onclick="document.ookieLeaderboard.cycleInviteButton();">change invite code</a>
			`)+`
			</div>
		`);
		if (scrollTop != null) l('leaderboardTabPage').scrollTop = scrollTop;
	},


	init: function() {
		// If we don't have mod data yet (first run) then the load() function won't run.
		// Hook the load function to simplify ourself.
		Game.original_loadModData = Game.loadModData;
		Game.loadModData = () => {
			Game.loadModData = Game.original_loadModData;
			Game.original_loadModData();
			if (this && !this.settings) this.load();
		};
		// Deleting mod data doesn't work if the plugin is still loaded/enabled.
		// Delete mod data -> save & quit -> mod saves data again -> oops...
		Game.original_deleteModData = Game.deleteModData;
		Game.deleteModData = (id) => {
			Game.original_deleteModData(id);
			if (id == "ookieLeaderboard") this.initDatas(null);
		};
		Game.original_deleteAllModData = Game.deleteAllModData;
		Game.deleteAllModData = () => {
			Game.original_deleteAllModData();
			this.initDatas(null);
		};

		document.ookieLeaderboard = this;//bleh
		this.updateS = this.dev ? 5 : 30;
		this.queryS = this.dev ? 5 : 60;
		if (this.dev) this.baseURL = "http://127.0.0.1:12345/er";
		else this.baseURL = "https://c.ookie.click/er";

		document.head.appendChild(document.createElement("style")).innerHTML = `
			#ookieLeaderboard {
				//background: url(img/starbg.jpg);
				background: url(img/shipmentBackground.png);
				background-size: auto 100%;
			}
			#leaderboardTabBar {
				display: flex;
				justify-content: center;
			}
			#leaderboardTitle {
				//margin-top: 0px;
				font-size: 20px;
			}
			.leaderboardTab {
				flex: 1;
				//padding: 1em;
				text-align: center;
				margin: 13px 10px 5px 10px;
				background: #000;
				max-width: 33%;
				font-size: 16px;
				color: white;
				background: url(img/darkNoise.jpg);
				overflow: hidden;
			}
			.leaderboardTabSelected {
				color: black;
				background: url(img/mineBackground.png);
			}
			#leaderboardTabPage {
				//background: url(img/shipmentBackground.png);
				border-radius: 25px 25px 0px 0px;
				padding-top: 1px;
				overflow-y: auto;
				overscroll-behavior-y: contain;
				max-height: 300px;
			}
			#leaderboardTabPage > table {
				margin: 20px 20px 20px 20px;
				counter-reset: rowNumber;
			}
			#leaderboardTabPage > table tr > td:first-child {
				counter-increment: rowNumber;
			}
			#leaderboardTabPage > table tr td:first-child::before {
				content: counter(rowNumber, decimal-leading-zero) ' ';
				//float: right;
				min-width: 1em;
				margin-right: 0.5em;
			}
			/*.leaderboardTabPageTBodyButImOwner > tr::after {
				content: "(kick)";
				min-width: 1em;
				margin-right: 0.5em;
			}*/
			#leaderboardTabPageTable > th {
				//width: 33%;
				text-align: left;
				text-decoration: underline;
			}
			#leaderboardTabPageTBody {
				margin-top: 22px;
			}
			#leaderboardTabPageTBody:before {
				line-height: 1em;
				content: "\\200C";
				display: block;
			}
			#leaderboardTabPageTBody > tr > td {
				padding-top: 4px;
				padding-bottom: 4px;
			}
			#leaderboardTabPageTBody > td {
				//background: #fff;
				//float: right;
				overflow: hidden;
			}
			.uncheckedName {
				filter: blur(8px) !important;
			}
			.uncheckedName:hover {
				filter: blur(0) !important;
			}
		`;
		l('buildingsMaster').insertAdjacentHTML('afterend', `
			<div id="ookieLeaderboard" class="row enabled">
				<!--
				<div id="leaderboardTitle" class="inset title zoneTitle">
					c.ookie.click/er/ leaderboard
				</div>
				-->
				<div id="leaderboardTabBar">
					loading...
				</div>
				<div class="separatorBottom"></div>
				<div class="productButtons" id="leaderboardProducts">
					<!--<div class="productButton" id="leaderboardChat" onclick="document.ookieLeaderboard.settingsMenu()">
						settings
					</div>-->
					<div class="productButton" id="leaderboardProductJoinCreate" onclick="document.ookieLeaderboard.joinCreatePrompt()">
						${this.dev?"(DEV ENABLED) ":""}join/create leaderboard
					</div>
					<div class="productButton" id="leaderboardProductGlobal" onclick="document.ookieLeaderboard.joinGlobalPrompt()">
						join global
					</div>
					<div class="productButton" id="leaderboardProductMod" onclick="document.ookieLeaderboard.openModWindow()" style="display: none;">
						mod👑
					</div>
					<!--<div class="productButton" id="leaderboardChat" onclick="document.ookieLeaderboard.openChat()">
						live-chat
					</div>-->
				</div>
			</div>
		`);
	},
	rateLimit: function(t, key) {
		const now = new Date()/1000;
		if ((now-this["____last_"+key])<5) return true;
		this["____last_"+key] = now;
		return false;
	},
	escapeHTML: function(s) {
		return s.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&apos;");
	},
	isOkayName: function(s) {
		const b = (new TextEncoder()).encode(s);
		if (b.length < 1 && b.length > 31) return false;
		for (const c of s)
			if (c < 0x20)
				return false;
		return true;
	},
});
