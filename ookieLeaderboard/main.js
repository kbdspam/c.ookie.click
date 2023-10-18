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
		}).then(response => {});
	},
	addTabBar: function() {
		if (document.getElementById("leaderboardTabBar")) document.getElementById("leaderboardTabBar").remove();
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
		if (document.getElementById("leaderboardTabPage")) document.getElementById("leaderboardTabPage").remove();
		l('leaderboardTabBar').insertAdjacentHTML('afterend', `
			<div id="leaderboardTabPage">
				test
			</div>
		`);
	},
	init: function() {
		let MOD = this;
		this.updatemeInterval = setInterval(function(){MOD.leaderboard_updateme();}, 5*1000);
		this.queryInterval = setInterval(function(){MOD.leaderboard_query();}, 5*1000);

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
		this.addTabPage();
	},
});
