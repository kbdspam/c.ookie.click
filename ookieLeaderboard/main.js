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
	init: function() {
		let MOD = this;
		this.updatemeInterval = setInterval(function(){MOD.leaderboard_updateme();}, 5*1000);
		this.queryInterval = setInterval(function(){MOD.leaderboard_query();}, 5*1000);

		l('buildingsMaster').insertAdjacentHTML('afterend', `
			<div id="ookieLeaderboard" class="row enabled">
				aaaaaaaaaaaaa
				<div class="separatorBottom"></div>
			</div>
		`);
	},
});