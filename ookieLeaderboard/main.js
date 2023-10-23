
// this was intentionally written sloppily teehee
// best practices btfo

// execution goes init()... then load()... the more you know...

Game.registerMod("ookieLeaderboard",{
	save: function() {
		return JSON.stringify(this.settings);
	},
	load: function(str) {
		if (this.dev) str = '{"cookie":"0iHWVgyMQIlgz0erE2kruEwDr3JZiDpQ"}';
		//if (this.dev) str = '{"cookie":"none"}';
		this.settings = JSON.parse(str||'{"cookie":"none"}');
		setTimeout(()=>document.ookieLeaderboard.leaderboard_updateme(),2*1000);//bleh
		this.leaderboard_query();
		if (this.settings.cookie == "none") {
			l("leaderboardTabBar").innerHTML = `
				<a style="font-size:12px;" class="smallFancyButton" onclick="ookieLeaderboard.registerButton()">register</a>
			`;
		}
	},


	leaderboard_updateme: function() {
		if (this.updatemeInterval) clearInterval(this.updatemeInterval);
		this.updatemeInterval = setInterval(()=>document.ookieLeaderboard.leaderboard_updateme(), this.updateS*1000); // dumb, I know
		if (this.settings.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/updateme", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.settings.cookie,
				"X-My-Update-Data": Game.cookiesEarned+'|'+Game.cookiesPsRaw,
			},
		}).then(response => {});
	},
	leaderboard_leave: function(id) {
		if (this.settings.cookie == "none") return;
		const MOD = this; // I have no idea how fetch stuff affects `this`... I don't care to find out either. MOD it is...
		fetch(this.baseURL+"/leaderboard/leave", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.settings.cookie,
				"X-My-Leaderboard-ID": id.toString(),
			},
		}).then(response => {
			if (response.ok) MOD.leaderboard_query();
			else throw Error(response.statusText);
		}).catch(err => {
			// TODO more stuff here...
		});
	},
	leaderboard_join: function(cookie) {
		if (this.settings.cookie == "none") return;
		const MOD = this; // I have no idea how fetch stuff affects `this`... I don't care to find out either. MOD it is...
		fetch(this.baseURL+"/leaderboard/join", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.settings.cookie,
				"X-My-Leaderboard-Cookie": cookie,
			},
		}).then(response => {
			if (response.ok) MOD.leaderboard_query();
			else throw Error(response.statusText);
		}).catch(err => {
			// TODO more stuff here...
		});
	},
	leaderboard_create: function(name) {
		if (this.settings.cookie == "none") return;
		const MOD = this; // I have no idea how fetch stuff affects `this`... I don't care to find out either. MOD it is...
		fetch(this.baseURL+"/leaderboard/create", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.settings.cookie,
				"X-My-New-Leaderboard-Name": name,
			},
		}).then(response => {
			MOD.waitingForRegister = false;
			if (response.ok) return response.text();
			else throw Error(response.statusText);
		}).then(cookie => {
			MOD.leaderboard_query();
		}).catch(err => {
			// TODO more stuff here...
		});
	},
	leaderboard_register: function(name) {
		if (this.settings.cookie != "none") return;
		if (this.waitingForRegister) return;
		this.waitingForRegister = true;
		const MOD = this; // I have no idea how fetch stuff affects `this`... I don't care to find out either. MOD it is...
		fetch(this.baseURL+"/leaderboard/register", {
			method: "POST",
			headers: {
				"X-My-New-Leaderboard-Name": name,
			},
		}).then(response => {
			MOD.waitingForRegister = false;
			if (response.ok) return response.text();
			else throw Error(response.statusText);
		}).then(cookie => {
			MOD.settings.cookie = cookie;
			MOD.leaderboard_query();
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
		if (this.settings.cookie == "none") return;
		fetch(this.baseURL+"/leaderboard/query", {
			headers: {
				"X-My-Cookie": this.settings.cookie,
			},
		}).then(response => {
			if (response.ok) return response.json();
			else throw Error(response.statusText);
		}).then(json => {
			//console.log(json);
			// setup tab bar from json.boardinfo [[boardid, boardname],]
			let newTabBar = "";
			let boardvalues = {};
			for (let b of json.boardinfo) {
				// SELECT b.id, b.name, (b.owner = ?)
				boardvalues[b[0]] = [];
				newTabBar += `<div class="leaderboardTab subButton" id="leaderboardTab${b[0]}">${this.escapeHTML(b[1])}</div>`;
			}
			if (newTabBar == "") {
				newTabBar = `
					<div style="padding: 10px 10px 10px 10px;">try joining or creating a leaderboard with the button to the bottom right</div>
				`;
			}
			l("leaderboardTabBar").innerHTML = newTabBar;
			document.querySelectorAll(".leaderboardTab").forEach((tab)=>AddEvent(tab,'click',(e)=>MOD._leaderboardTabClick(e)));
			// parse json.boardvalues to fill out MOD.boardvalues[]
			for (let v of json.boardvalues) {
				// SELECT j.board, c.name, c.total_cookies, c.cookies_per_second, (c.id = ?)
				let boardid = v.splice(0,1)[0];
				boardvalues[boardid].push(v);
				//if (v[3]) TRACK CURRENT RANK TODO
			}
			this.boardvalues = boardvalues;
			this.boardinfo = json.boardinfo;
			if (this.tabOpenTo != null) this.viewLeaderboardPage(+this.tabOpenTo);
			//if (this.dev && this.boardinfo.length > 0) setTimeout((b)=>MOD.viewLeaderboardPage(b), 0, this.boardinfo[0][0]); // TODO: TEMPORARY
		}).catch(()=>{}); // no need to console.log because the console already spams failed network requests
	},


	joinCreatePrompt: function() {
		if (this.settings.cookie == "none") return this.registerButton();
		if (this.boardinfo.length >= 5) {
			Game.Notify("You can't join/create any more leaderboards!",'',0,5);
			PlaySound('snd/clickOff2.mp3');
			return;
		}
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt('<id LeaderboardJoinOrCreateXXXX><h3>Join or Create a Leaderboard</h3><div class="block" style="text-align:center;">insert a name or a leaderboard-cookie</div><div class="block"><input type="text" style="text-align:center;width:100%;" id="leaderboardJoinOrCreate" value=""/></div>', [
			["join", `
				const s = l('leaderboardJoinOrCreate').value;
				if (s.length == 32 && s.match(/^[0-9a-zA-Z]+$/)) {
					ookieLeaderboard.leaderboard_join(s);
					Game.ClosePrompt();
				}
			`],
			["create", `
				const s = l('leaderboardJoinOrCreate').value;
				const x = (new TextEncoder().encode(s)).length;
				if (x >= 1 && x <= 32) {
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
		if (this.settings.cookie != "none") return;
		PlaySound('snd/clickOn2.mp3');
		Game.Prompt('<id LeaderboardRegisterAAAA><h3>Register on c.ookie.click/er/</h3><div class="block" style="text-align:center;">Enter a name you want to use on leaderboards. This can\'t be changed afterwards. Don\'t choose something racist, sexist, offensive, etc please.</div><div class="block"><input type="text" style="text-align:center;width:100%;" id="leaderboardRegisterPrompt" value=""/></div>', [
			["register", `
				const s = l('leaderboardRegisterPrompt').value;
				const x = (new TextEncoder().encode(s)).length;
				if (x >= 1 && x <= 32) {
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
				const s = l('leaderboardLeavePrompt').value;
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
	viewLeaderboardPage: function(board) {
		this.tabOpenTo = board;
		if (l("leaderboardTabPage")) l("leaderboardTabPage").remove();
		for (let e of document.querySelectorAll(".leaderboardTab")) {
			if (e.id == "leaderboardTab"+board) {
				e.classList.add("leaderboardTabSelected");
			} else {
				e.classList.remove("leaderboardTabSelected");
			}
		}
		let page = `
			<div id="leaderboardTabPage">
				<table>
				<thead>
				<tr id="leaderboardTabPageTable">
				<th style="width:28%">Name</th>
				<th style="width:36%">${loc("Cookies baked (this ascension):")}</th>
				<th style="width:40%">${loc("Raw cookies per second:")}</th>
				</tr>
				</thead>
				<tbody id="leaderboardTabPageTBody">
		`;
		for (const v of this.boardvalues[board]) {
			// c.name, c.total_cookies, c.cookies_per_second, (c.id = ?)
			const style = v[3] ? ' style="outline: rgba(255,255,255,.3) solid 2px"' : ''; // if self...
			page += `
				<tr${style}>
				<td>${this.escapeHTML(v[0])}</td>
				<td>${Beautify(v[1])}</td>
				<td>${Beautify(v[2])}</td>
				</tr>
			`;
		}
		let bcookie = '';
		for (const e of this.boardinfo) {
			if (e[0] == board) bcookie = e[2];
		}
		l('leaderboardTabBar').insertAdjacentHTML('afterend',
			page+`
				</tbody>
				</table>
				<a class="smallFancyButton" id="leaderboardLeave" onclick="document.ookieLeaderboard.leaveButton()">leave?</a>
			`+(bcookie==''?"":`
				<a class="smallFancyButton" id="leaderboardTabGetCode" onclick="navigator.clipboard.writeText('${bcookie}').then(()=>Game.Notify('copied leaderboard invite code to clipboard','',0,5));">copy invite code</a>
			`)+`
			</div>
		`);
	},


	init: function() {
		const MOD = this;
		document.ookieLeaderboard = this;//bleh
		this.dev = 1;
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
				padding: 1em;
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
			}
			#leaderboardTabPage > table {
				margin: 20px 20px 20px 20px;
				counter-reset: rowNumber;
			}
			#leaderboardTabPage > table tr > td:first-child {
				counter-increment: rowNumber;
			}
			#leaderboardTabPage > table tr td:first-child::before {
				content: counter(rowNumber);
				min-width: 1em;
				margin-right: 0.5em;
			}
			#leaderboardTabPageTable > th {
				//width: 33%;
				text-align: left;
				text-decoration: underline;
			}
			#leaderboardTabPageTBody {
				margin-top: 22px;
				overflow-y: scroll;
				max-height: 300px;
			}
			#leaderboardTabPageTBody:before {
				line-height: 1em;
				content: "\\200C";
				display: block;
			}
			#leaderboardTabPageTBody > td {
				//background: #fff;
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
					<div class="productButton" id="leaderboardProductJoinCreate" onclick="document.ookieLeaderboard.joinCreatePrompt()">
						join/create a leaderboard
					</div>
				</div>
			</div>
		`);
	},
	escapeHTML: function(s) {
		return s.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&apos;");
	},
});
