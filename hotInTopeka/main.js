Game.registerMod("hotInTopeka",{
	init:function(){
		let MOD = this;
		MOD.isItHot = function(temperature) {
			if (temperature >= 90) {
				return "HOT";
			} else if (temperature >= 70) {
				return "MILD";
			} else if (temperature >= 62) {
				return "PERFECT";
			} else {
				return "COLD";
			}
		};
		MOD.updateTheThing = function() {
			fetch("https://c.ookie.click/er/topeka")
				.then(response => {
					if (response.ok) {
						l('hotInTopeka').innerHTML = "Topeka?<br>"+MOD.isItHot(response.text);
					}
				});
		};
		//Game.Notify(`Is it hot in Topeka?!`,`Now with extra clickable stuff!`,[16,5]);
		l('storeTitle').insertAdjacentHTML('afterbegin','<a style="font-size:12px;bottom:2px;left:4px;position:absolute;" class="smallFancyButton" id="hotInTopeka">Topeka?<br>???</a>');
		AddEvent(l('hotInTopeka'),'click',function(){
			Steam.openLink("https://www.youtube.com/watch?v=L_IlsPypwZs");
		});

		MOD.updateTheThing();
		setTimeout(MOD.updateTheThing, 120*1000);
	},
	save:function(){
		//use this to store persistent data associated with your mod
		//note: as your mod gets more complex, you should consider storing a stringified JSON instead
		return "";
	},
	load:function(str){
		//do stuff with the string data you saved previously
	},
});