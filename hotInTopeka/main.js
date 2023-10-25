Game.registerMod("hotInTopeka", {
	init: function() {
		l('storeTitle').insertAdjacentHTML('afterbegin','<a style="font-size:12px;bottom:2px;left:4px;position:absolute;" class="smallFancyButton" id="hotInTopeka">Topeka?<br>???</a>');
		AddEvent(l('hotInTopeka'),'click',()=>{
			PlaySound('snd/clickOn2.mp3');
			this.show_text = false;
			if (this.text_timeout != null) clearInterval(this.text_timeout);
			this.text_timeout = setTimeout(()=>{
				this.show_text = true;
				this.displayThing(false);
				PlaySound('snd/clickOff2.mp3');
			}, 2*1000);
			this.displayThing(false);
		});
		this.show_text = true;
		this.updateTheThing();
		setInterval(()=>{this.updateTheThing();}, 120*1000);
	},
	isItHot: function(temperature, notif) {
		if (temperature >= 90) {
			if (this.last == "MILD" && notif) {
				//Game.Notify(`It's hot in Topeka!`,`Wow!`,[16,5]);
			}
			return "HOT";
		} else if (temperature >= 80) {
			return "WARMER";
		} else if (temperature >= 70) {
			return "WARM";
		} else if (temperature >= 60) {
			return "NICE";
		} else if (temperature >= 48) {
			return "COLD";
		} else {
			return "FREEZING";
		}
	},
	displayThing: function(notif) {
		if (this.temp == null) {
			l('hotInTopeka').innerHTML = "Topeka?<br>???";
			return;
		}
		this.last = this.isItHot(this.temp, notif);
		l('hotInTopeka').innerHTML = "Topeka?<br>"+(this.show_text?this.last:this.temp);
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
				this.temp = +response;
				this.displayThing(true);
			})
			.catch(err => console.log("hotInTopeka error: "+err));
	},
	save:function() {
		return "";
	},
	load:function(str) {
	},
});