// SPDX-License-Identifier:
// Copyright

//#![forbid(unsafe_code)]

use std::path::PathBuf;

use tokio::net::UnixListener;

mod cursor_party;
mod leaderboard;
mod wstimer;

#[allow(dead_code)]
async fn get_uds(path: PathBuf) -> anyhow::Result<UnixListener> {
	let _ = tokio::fs::remove_file(&path).await;
	tokio::fs::create_dir_all(path.parent().unwrap()).await?;
	Ok(UnixListener::bind(path)?)
}

fn main() -> anyhow::Result<()> {
	unsafe {
		std::env::set_var("RUST_BACKTRACE", "full");
	}

	//tracing_subscriber::fmt::init();

	tokio::runtime::Runtime::new().unwrap().block_on(async {
		let mut tasks = tokio::task::JoinSet::new();
		tasks.spawn(leaderboard::run());
		tasks.spawn(cursor_party::run());

		while let Some(t) = tasks.join_next().await {
			t??;
		}

		Ok(())
	})
}
