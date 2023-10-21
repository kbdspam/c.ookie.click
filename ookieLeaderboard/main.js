/*

leaderboard tabs where the tab-title-text is your rank in the leaderboard
hover shows:
-------------
#1 blah  123123 cokies
#2 xyz     3233 cookies
#3 hbqq    1111 cookies
#33 (YOU)   999 cookies
-------------

*/

Game.registerMod("ookieLeaderboard",{
	save: function() {
		return JSON.stringify(this.settings);
	},
	load: function(str) {
		this.settings = JSON.parse(str||'{"cookie":"none"}');
	},
	leaderboard_updateme: function() {
		if (this.settings.cookie == "none") return;
		fetch("https://c.ookie.click/er/leaderboard/updateme", {
			method: "POST",
			headers: {
				"X-My-Cookie": this.settings.cookie,
				"X-My-Update-Data": Game.cookiesEarned+'|'+Game.cookiesPsRaw,
			},
		}).then(response => {});
	},
	leaderboard_query: function() {
		if (this.settings.cookie == "none") return;
		console.log(this.settings);
		fetch("https://c.ookie.click/er/leaderboard/query", {
			headers: {
				"X-My-Cookie": this.settings.cookie,
			},
		}).then(response => {
			if (response.ok)
				return response.json();
			else
				throw Error(response.statusText);
		})
		.then(json => {
			// setup tab bar from json.boardinfo [[boardid, boardname],]
			let newTabBar = "";
			let boardvalues = {};
			const sanitizer = new Sanitizer();
			for (let b of json.boardinfo) {
				// SELECT b.id, b.name
				boardvalues[b[0]] = [];
				newTabBar += `<div class="leaderboardTab subButton" id="leaderboardTab${b[0]}">${sanitizer.sanitizeFor("div", b[1])}</div>`;
			}
			if (json.boardinfo.length < 5)
				newTabBar += '<div class="leaderboardTab subButton" id="leaderboardTabNew">join/create a new board</div>';
			l("leaderboardTabBar").innerHTML = newTabBar;
			// parse json.boardvalues to fill out MOD.boardvalues[]
			for (let v of json.boardvalues) {
				// SELECT j.board, c.name, c.total_cookies, c.cookies_per_second, (c.id = ?)
				let boardid = v.splice(1)[0];
				boardvalues[boardid].push(v);
				//if (v[3]) TRACK CURRENT RANK TODO
			}
			this.boardvalues = boardvalues;
			this.boardinfo = json.boardinfo;
			if (this.boardinfo.length > 0) this.tabOpenTo = this.boardinfo[0][0]; // TODO: TEMPORARY
			let MOD = this;
			setTimeout(()=>MOD.addTabPage());
		})
		.catch(err => console.log("leaderboard_query error: "+err));
	},
	addTabBar: function() {
		if (l("leaderboardTabBar")) l("leaderboardTabBar").remove();
		l('leaderboardTitle').insertAdjacentHTML('afterend', `
			<div id="leaderboardTabBar">
				<div class="leaderboardTab subButton">asdf</div>
				<div class="leaderboardTab subButton">asdf</div>
				<div class="leaderboardTab subButton">asdf</div>
				<div class="leaderboardTab subButton">asdf</div>
				<div class="leaderboardTab subButton">asdf</div>
			</div>
		`);
	},
	addTabPage: function() {
		if (this.tabOpenTo == null) return;
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
		const sanitizer = new Sanitizer();
		for (let v of this.boardvalues[this.tabOpenTo]) {
			// c.name, c.total_cookies, c.cookies_per_second, (c.id = ?)
			const style = v[3] ? ' style="outline: #f00 solid 2px"'; // if self...
			page += `
				<tr${style}>
				<td>${sanitizer.sanitizeFor("td", v[0])}</td>
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
		let MOD = this;
		this.updatemeInterval = setInterval(()=>MOD.leaderboard_updateme(), 5*1000);
		this.queryInterval = setInterval(()=>MOD.leaderboard_query(), 5*1000);

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
		//this.addTabPage();
	},
});
