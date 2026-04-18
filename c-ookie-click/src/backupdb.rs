use std::{
	path::{Path, PathBuf},
	time::Duration,
};

async fn do_it(dbdir: &Path) -> anyhow::Result<()> {
	let backup_to_this = dbdir
		.join("backup")
		.join(format!("leaderboard_{}.db", jiff::Timestamp::now().as_second()));
	let _ = tokio::process::Command::new("sqlite3")
		.arg(&dbdir.join("leaderboard.db"))
		.arg(format!("VACUUM INTO '{}'", backup_to_this.to_string_lossy()))
		.spawn()?
		.wait()
		.await?;
	let _ = tokio::process::Command::new("gzip")
		.arg(&backup_to_this)
		.spawn()?
		.wait()
		.await?;
	Ok(())
}

pub(super) async fn run() -> anyhow::Result<()> {
	let dbdir = PathBuf::from(&std::env::var("DBPATH").unwrap_or_else(|_| "../data".to_owned()));
	tokio::fs::create_dir_all(dbdir.join("backup")).await?;

	loop {
		let _ = do_it(&dbdir).await;
		tokio::time::sleep(Duration::from_hours(1)).await;
	}
}
