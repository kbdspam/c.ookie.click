/*

design for max 100 online users.
send updates ever 30s? query every 60s?

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
	init:function() {
		let MOD = this;

	},
	save:function() {
		return JSON.stringify(this.settings);
	},
	load:function(str) {
		this.settings = JSON.parse(str||'{"cookie":""}');
	},
});