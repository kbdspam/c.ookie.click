
// this was intentionally written sloppily teehee
// best practices btfo

// execution goes init()... then load()... the more you know...

Game.registerMod("ookieLeaderboard",{
	save: function() {
		return JSON.stringify(this.settings);
	},
	load: function(str) {
		if (this.dev) str = '{"cookie":"0iHWVgyMQIlgz0erE2kruEwDr3JZiDpQ"}';
		this.settings = JSON.parse(str||'{"cookie":"none"}');
		setTimeout(()=>document.ookieLeaderboard.leaderboard_updateme(),2*1000);//bleh
		this.leaderboard_query();
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
		console.log("hello");
		if (this.settings.cookie == "none") return;
		const MOD = this; // I have no idea how fetch stuff affects `this`... I don't care to find out either. MOD it is...
		console.log("hello2");
		fetch(this.baseURL+"/leaderboard/create", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.settings.cookie,
				"X-My-New-Leaderboard-Name": name,
			},
		}).then(response => {
			console.log("hello3");
			MOD.waitingForRegister = false;
			if (response.ok) return response.text();
			else throw Error(response.statusText);
		}).then(cookie => {
			console.log("hello4");
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
			this.settings.cookie = cookie;
		}).catch(err => {
			// TODO more stuff here...
		});
	},
	_leaderboardTabClick: function(e) {
		const item = e.target.id.split("leaderboardTab")[1]; // this is so stupid but it's so easy
		if (this.tabOpenTo == item || this.tabOpenTo == +item) {
			this.tabOpenTo = null;
			PlaySound('snd/clickOff2.mp3');
			if (l("leaderboardTabPage")) l("leaderboardTabPage").remove();
			return;
		}
		PlaySound('snd/clickOn2.mp3');
		if (item != "New") this.viewLeaderboardPage(+item);
		else {
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
		}
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
			if (json.boardinfo.length < 5)
				newTabBar += '<div class="leaderboardTab subButton" id="leaderboardTabNew">join/create a new board</div>';
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
			//if (this.dev && this.boardinfo.length > 0) setTimeout((b)=>MOD.viewLeaderboardPage(b), 0, this.boardinfo[0][0]); // TODO: TEMPORARY
		}).catch(()=>{}); // no need to console.log because the console already spams failed network requests
	},
	addTabBar: function() {
		if (l("leaderboardTabBar")) l("leaderboardTabBar").remove();
		l('leaderboardTitle').insertAdjacentHTML('afterend', `
			<div id="leaderboardTabBar">
				loading...
			</div>
		`);
	},
	viewLeaderboardPage: function(board) {
		this.tabOpenTo = board;
		if (l("leaderboardTabPage")) l("leaderboardTabPage").remove();
		let page = `
			<div id="leaderboardTabPage">
				<table id="leaderboardTabPageTable">
				<thead>
				<tr>
				<th>Name</th>
				<th>${loc("Cookies baked (this ascension):")}</th>
				<th>${loc("Raw cookies per second:")}</th>
				</tr>
				</thead>
				<tbody id="leaderboardTabPageTBody">
		`;
		for (let v of this.boardvalues[board]) {
			// c.name, c.total_cookies, c.cookies_per_second, (c.id = ?)
			const style = v[3] ? ' style="outline: #f00 solid 2px"' : ''; // if self...
			page += `
				<tr${style}>
				<td>${this.escapeHTML(v[0])}</td>
				<td>${v[1]}</td>
				<td>${v[2]}</td>
				</tr>
			`;
		}
		l('leaderboardTabBar').insertAdjacentHTML('afterend',
			page+`
				</tbody>
				</table>
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
				background:url(img/starbg.jpg);
			}
			#leaderboardTabBar {
				display: flex;
			}
			.leaderboardTab {
				flex: 1;
				padding: 1em;
				text-align: center;
				margin: 10px 10px 5px 10px;
				background: #000;
			}
		`;

		l('buildingsMaster').insertAdjacentHTML('afterend', `
			<div id="ookieLeaderboard" class="row enabled">
				<div id="leaderboardTitle">Leaderboard...</div>
				<div class="separatorBottom"></div>
			</div>
		`);
		this.addTabBar();
	},
	escapeHTML: function(s) {
		return s.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&apos;");
	},
});
