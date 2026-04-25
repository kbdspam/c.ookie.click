// $ units "tempC($(curl -s "https://api.weather.gov/stations/KTOP/observations/latest" | jq ."properties.temperature.value"))" "tempF" | xargs

use std::{path::PathBuf, time::Duration};

use serde::Deserialize;

#[derive(Deserialize)]
struct Observation {
	properties: Properties,
}
#[derive(Deserialize)]
struct Properties {
	temperature: Temperature,
}
#[derive(Deserialize)]
struct Temperature {
	value: f64,
}

async fn do_it() -> anyhow::Result<f64> {
	let output = tokio::process::Command::new("curl")
		.args(&["-s", "https://api.weather.gov/stations/KTOP/observations/latest"])
		.output()
		.await?;
	let j: Observation = serde_json::from_slice(&output.stdout)?;
	Ok(j.properties.temperature.value * 1.8 + 32.0)
}

pub(super) async fn run() -> anyhow::Result<()> {
	let topeka_file = PathBuf::from(&std::env::var("PUBLICDIR").unwrap_or_else(|_| "../public".to_owned()))
		.join("c.ookie.click/er/topeka");
	loop {
		if let Ok(fahrenheit) = do_it().await {
			//println!("topeka = {fahrenheit:.1}F");
			tokio::fs::write(&topeka_file, format!("{fahrenheit}")).await?;
		}
		tokio::time::sleep(Duration::from_mins(10)).await;
	}
}
