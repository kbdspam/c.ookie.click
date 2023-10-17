Game.registerMod("hotInTopeka", {
	init: function() {
		let MOD = this;
		l('storeTitle').insertAdjacentHTML('afterbegin','<a style="font-size:12px;bottom:2px;left:4px;position:absolute;" class="smallFancyButton" id="hotInTopeka">Topeka?<br>???</a>');
		AddEvent(l('hotInTopeka'),'click',function(){
			Steam.openLink("https://www.youtube.com/watch?v=L_IlsPypwZs");
		});
		this.updateTheThing();
		setInterval(function() {MOD.updateTheThing();}, 120*1000);
	},
	isItHot: function(temperature) {
		if (temperature >= 90) {
			if (this.last == "MILD") {
				//Game.Notify(`It's hot in Topeka!`,`Wow!`,[16,5]);
			}
			return "HOT";
		} else if (temperature >= 70) {
			return "MILD";
		} else if (temperature >= 62) {
			return "NICE";
		} else if (temperature >= 45) {
			return "COLD";
		} else {
			return "FREEZING";
		}
	},
	updateTheThing: function() {
		fetch("https://c.ookie.click/er/topeka")
			.then(response => {
				if (response.ok)
					return response.text();
				else
					throw Error(response.statusText);
			})
			.then(response => {
				this.last = this.isItHot(+response);
				l('hotInTopeka').innerHTML = "Topeka?<br>"+this.last;
			})
			.catch(err => console.log("hotInTopeka error: "+err));
	},
	save:function() {
		return "";
	},
	load:function(str) {
	},
});