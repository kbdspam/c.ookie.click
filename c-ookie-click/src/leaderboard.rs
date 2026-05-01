// SPDX-License-Identifier:
// Copyright

// TODO: rate-limiting

use std::{path::PathBuf, sync::Arc};

use axum::{
	Router,
	extract::State,
	http::{HeaderMap, StatusCode},
	response::Redirect,
	routing::{any, get, post},
};
use itertools::Itertools;
use rand::{RngExt, distr::Distribution};
use sqlx::{
	AssertSqlSafe,
	sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use tower_http::services::ServeDir;

struct AxumState {
	pool: sqlx::Pool<sqlx::Sqlite>,
}

#[derive(sqlx::FromRow)]
struct ClickersRow {
	id: i64,
	can_mod: i64,
	name: Option<String>,
}

pub(super) async fn run() -> anyhow::Result<()> {
	let dbpath =
		PathBuf::from(&std::env::var("DBPATH").unwrap_or_else(|_| "../data".to_owned())).join("leaderboard.db");

	let pool = SqlitePoolOptions::new()
		.max_connections(10)
		.min_connections(2)
		.connect_with(
			SqliteConnectOptions::new()
				.create_if_missing(true)
				.journal_mode(SqliteJournalMode::Wal)
				.synchronous(SqliteSynchronous::Extra)
				.filename(dbpath),
		)
		.await?;

	// https://docs.rs/sqlx/latest/sqlx/migrate/trait.MigrationSource.html
	sqlx::migrate!().run(&pool).await?;

	let state = Arc::new(AxumState { pool });

	let dir =
		PathBuf::from(&std::env::var("PUBLICDIR").unwrap_or_else(|_| "../public".to_owned())).join("c.ookie.click");

	// TODO: tracing
	// TODO: tower_http::sensitive_headers
	let app = Router::new()
		.route("/", get(|| async { Redirect::permanent("/er/") }))
		.fallback_service(ServeDir::new(dir))
		.route("/er/leaderboard/register", post(leaderboard_register))
		.route("/er/leaderboard/changemyname", post(leaderboard_changemyname))
		.route("/er/leaderboard/create", post(leaderboard_create))
		.route("/er/leaderboard/cycleboardcookie", post(leaderboard_cycleboardcookie))
		.route("/er/leaderboard/changeboardname", post(leaderboard_changeboardname))
		.route("/er/leaderboard/kick", post(leaderboard_kick))
		.route("/er/leaderboard/updateme", post(leaderboard_updateme))
		.route("/er/leaderboard/query", get(leaderboard_query))
		.route("/er/leaderboard/leave", post(leaderboard_leave))
		.route("/er/leaderboard/join", post(leaderboard_join))
		.route("/er/leaderboard/wstimer", any(crate::wstimer::handler))
		.with_state(state);

	let mut tasks = tokio::task::JoinSet::new();

	tasks.spawn({
		let app = app.clone();
		async move { anyhow::Ok(axum::serve(tokio::net::TcpListener::bind("0.0.0.0:8080").await?, app).await?) }
	});
	tasks.spawn(
		async move { Ok(axum::serve(crate::get_uds("/tmp/c.ookie.click/main.sock".into()).await?, app).await?) },
	);

	while let Some(t) = tasks.join_next().await {
		t??;
	}

	Ok(())
}

fn randcookie() -> String {
	struct CapitalAndLowercaseHexadecimal;
	impl Distribution<u8> for CapitalAndLowercaseHexadecimal {
		fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> u8 {
			const S: &[u8] = b"0123456789abcdefABCDEF";
			let i = rng.random_range(0..S.len());
			S[i]
		}
	}
	let mut rng = rand::rng();
	(0..32)
		.map(|_| rng.sample(CapitalAndLowercaseHexadecimal) as char)
		.collect()
}

async fn disabled_registering() -> Result<(), (StatusCode, String)> {
	let path =
		PathBuf::from(&std::env::var("DBPATH").unwrap_or_else(|_| "../data".to_owned())).join("disabled_registering");
	if tokio::fs::try_exists(path).await.unwrap_or(false) {
		Ok(())
	} else {
		Err((StatusCode::INTERNAL_SERVER_ERROR, "db broken".to_owned()))
	}
}

async fn disabled_leaderboard_create() -> Result<(), (StatusCode, String)> {
	let path = PathBuf::from(&std::env::var("DBPATH").unwrap_or_else(|_| "../data".to_owned()))
		.join("disabled_leaderboard_create");
	if tokio::fs::try_exists(path).await.unwrap_or(false) {
		Ok(())
	} else {
		Err((StatusCode::INTERNAL_SERVER_ERROR, "db broken".to_owned()))
	}
}

fn check_for_bad_workshop_id(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
	// TODO:
	if true {
		return Ok(());
	}

	let workshop_id = get_header(headers, "X-My-Workshop-ID")?;
	if jiff::Timestamp::now().as_second() > 1724017056 && workshop_id == "3061304069" {
		// Sun Aug 18 2024 21:37:36 GMT+0000
		Ok(())
	} else {
		Err((StatusCode::INTERNAL_SERVER_ERROR, "db broken".to_owned()))
	}
}

fn get_header<'a, 'b>(headers: &'a HeaderMap, header_name: &'b str) -> Result<&'a str, (StatusCode, String)> {
	let Some(header) = headers.get(header_name) else {
		return Err((StatusCode::BAD_REQUEST, format!("missing header {header_name}")));
	};
	let Ok(header) = header.to_str() else {
		return Err((StatusCode::BAD_REQUEST, format!("bad header {header_name}")));
	};
	Ok(header)
}

fn get_name(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
	let name = get_header(headers, "X-My-New-Leaderboard-Name")?;
	let name = name.trim();
	fn is_okay_name(s: &str) -> bool {
		s.len() > 0 && s.len() < 32 && s.chars().all(|c| c >= 0x20 as char)
	}
	if !is_okay_name(name) {
		return Err((StatusCode::BAD_REQUEST, "name too big or too small".to_owned()));
	}
	Ok(name.to_owned())
}

fn get_cookie(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
	let cookie = get_header(headers, "X-My-Cookie")?;
	if cookie.len() != 32 {
		return Err((StatusCode::BAD_REQUEST, "cookie len != 32".to_owned()));
	}
	Ok(cookie.to_string())
}

fn get_leaderboard_id(headers: &HeaderMap) -> Result<i64, (StatusCode, String)> {
	let boardid = get_header(headers, "X-My-Leaderboard-ID")?;
	let Ok(boardid) = boardid.parse::<i64>() else {
		return Err((StatusCode::BAD_REQUEST, "bad boardid 2".to_owned()));
	};
	Ok(boardid)
}

async fn leaderboard_register(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	disabled_registering().await?;
	let name = get_name(&headers)?;
	let cookie = randcookie();

	sqlx::query("INSERT INTO clickers(name, cookie) VALUES ($1,$2);")
		.bind(&name)
		.bind(&cookie)
		.execute(&state.pool)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	// disabled global-leaderboard joining
	//cur.execute("INSERT INTO joinedboards(clicker, board) VALUES (?,1);", (cur.last_insert_rowid,))

	Ok(cookie)
}

async fn leaderboard_changemyname(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	disabled_registering().await?;
	let name = get_name(&headers)?;
	let cookie = get_cookie(&headers)?;

	let rows_affected = sqlx::query(
		"
		UPDATE clickers SET
		name=$1,
		last_upodated=unixepoch(),
		okay_name=(CASE okay_name WHEN -2 THEN -2 ELSE 0 END)
		WHERE cookie = $2
	",
	)
	.bind(name)
	.bind(cookie)
	.execute(&state.pool)
	.await
	.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
	.rows_affected();

	if rows_affected > 0 {
		// TODO: Ping me on discord to whitelist...  Actually, not discord, but something eventually maybe...
		Ok("changed".to_owned())
	} else {
		Err((StatusCode::INTERNAL_SERVER_ERROR, "no?".to_owned()))
	}
}

async fn leaderboard_create(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	disabled_leaderboard_create().await?;
	let name = get_name(&headers)?;
	let cookie = get_cookie(&headers)?;
	let boardcookie = randcookie();

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	let cid = sqlx::query_scalar::<_, i64>("SELECT id FROM clickers WHERE cookie = $1")
		.bind(&cookie)
		.fetch_optional(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
		.map_or_else(
			|| {
				Err((
					StatusCode::UNAUTHORIZED,
					"no clicker exists with that cookie".to_owned(),
				))
			},
			Ok,
		)?;

	let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joinedboards WHERE clicker = $1")
		.bind(cid)
		.fetch_one(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	if count >= 5 {
		return Err((StatusCode::FORBIDDEN, "in too many boards".to_owned()));
	}

	sqlx::query(
		"INSERT INTO boards(name, owner, cookie, only_owner_cookie, last_updated) VALUES ($1,$2,$3,0,unixepoch());",
	)
	.bind(name)
	.bind(cid)
	.bind(boardcookie)
	.execute(&mut *db)
	.await
	.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;
	// <splitting these queries because idk if last_insert_rowd() works in a transaction>
	sqlx::query("INSERT INTO joinedboards(clicker, board) VALUES (?, last_insert_rowid());")
		.bind(cid)
		.execute(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	Ok("".to_owned())
}

async fn leaderboard_cycleboardcookie(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let boardid = get_leaderboard_id(&headers)?;

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	let clickerid = sqlx::query_scalar::<_, i64>("SELECT id FROM clickers WHERE cookie = $1")
		.bind(&cookie)
		.fetch_optional(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
		.map_or_else(
			|| {
				Err((
					StatusCode::UNAUTHORIZED,
					"no clicker exists with that cookie".to_owned(),
				))
			},
			Ok,
		)?;

	let rows_affected =
		sqlx::query("UPDATE boards SET cookie = $1, last_updated=unixepoch() WHERE owner = $2 AND id = $3")
			.bind(randcookie())
			.bind(clickerid)
			.bind(boardid)
			.execute(&mut *db)
			.await
			.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
			.rows_affected();

	if rows_affected > 0 {
		Ok("cycled".to_owned())
	} else {
		Err((StatusCode::BAD_REQUEST, "no?".to_owned()))
	}
}

async fn leaderboard_changeboardname(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let name = get_name(&headers)?;
	let boardid = get_leaderboard_id(&headers)?;

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	let clickerid = sqlx::query_scalar::<_, i64>("SELECT id FROM clickers WHERE cookie = $1")
		.bind(&cookie)
		.fetch_optional(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
		.map_or_else(
			|| {
				Err((
					StatusCode::UNAUTHORIZED,
					"no clicker exists with that cookie".to_owned(),
				))
			},
			Ok,
		)?;

	let rows_affected =
		sqlx::query("UPDATE boards SET name = $1, last_updated=unixepoch() WHERE owner = $2 AND id = $3")
			.bind(name)
			.bind(clickerid)
			.bind(boardid)
			.execute(&mut *db)
			.await
			.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
			.rows_affected();

	if rows_affected > 0 {
		Ok("changed".to_owned())
	} else {
		Err((StatusCode::BAD_REQUEST, "no?".to_owned()))
	}
}

async fn leaderboard_kick(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let enemy = get_header(&headers, "X-My-Enemy-ID")?
		.parse::<i64>()
		.map_err(|_| (StatusCode::BAD_REQUEST, "bad header X-My-Enemy-ID".to_owned()))?;
	let boardid = get_leaderboard_id(&headers)?;

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	let clicker = sqlx::query_as::<_, ClickersRow>("SELECT id, can_mod FROM clickers WHERE cookie = $1")
		.bind(&cookie)
		.fetch_optional(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
		.map_or_else(
			|| {
				Err((
					StatusCode::UNAUTHORIZED,
					"no clicker exists with that cookie".to_owned(),
				))
			},
			Ok,
		)?;

	if enemy == clicker.id {
		return Err((StatusCode::BAD_REQUEST, "no...".to_owned()));
	}

	let can_kick;

	if boardid == 1 {
		can_kick = clicker.can_mod >= 2;
	} else {
		let rows_affected = sqlx::query("UPDATE boards SET cookie = $1 WHERE owner = $2 AND id = $3")
			.bind(randcookie())
			.bind(clicker.id)
			.bind(boardid)
			.execute(&mut *db)
			.await
			.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
			.rows_affected();
		can_kick = rows_affected > 0;
	}

	if can_kick {
		let rows_affected = sqlx::query("DELETE FROM joinedboards WHERE clicker = $1 AND board = $2")
			.bind(clicker.id)
			.bind(boardid)
			.execute(&mut *db)
			.await
			.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
			.rows_affected();
		if rows_affected > 0 {
			let _ = sqlx::raw_sql(AssertSqlSafe(format!(
				"
				UPDATE clickers SET last_updated=unixepoch() WHERE id={enemy};
				UPDATE boards SET last_updated=unixepoch() WHERE id={boardid};
				"
			)))
			.execute(&mut *db)
			.await;
			Ok("kicked".to_owned())
		} else {
			Err((StatusCode::BAD_REQUEST, "not in board?".to_owned()))
		}
	} else {
		Err((StatusCode::BAD_REQUEST, "no dice".to_owned()))
	}
}

async fn leaderboard_updateme(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let data = get_header(&headers, "X-My-Update-Data")?;
	let (total_cookies, cookies_per_second) = data
		.split('|')
		.map(|s| s.parse::<f64>().unwrap_or(-1.0))
		.collect_tuple()
		.unwrap_or((-1.0, -1.0));

	if !total_cookies.is_finite() || total_cookies < 0.0 || !cookies_per_second.is_finite() || cookies_per_second < 0.0
	{
		let _ = sqlx::query("UPDATE clickers SET cheater=1, last_updated=unixepoch() WHERE cookie = $1")
			.bind(cookie)
			.execute(&state.pool)
			.await;
		return Err((StatusCode::INTERNAL_SERVER_ERROR, "".to_owned()));
	}

	let rows_affected = sqlx::query(
		"UPDATE clickers SET last_updated=unixepoch(), total_cookies = $1, cookies_per_second = $2 WHERE cookie = $3",
	)
	.bind(total_cookies)
	.bind(cookies_per_second)
	.bind(cookie)
	.execute(&state.pool)
	.await
	.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
	.rows_affected();

	if rows_affected > 0 {
		Ok("updated".to_owned())
	} else {
		Err((StatusCode::FORBIDDEN, "???".to_owned()))
	}
}

async fn leaderboard_query(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let mut timestamp = get_header(&headers, "X-My-Timestamp2")
		.unwrap_or("0")
		.parse::<i64>()
		.unwrap_or(0);
	let now = jiff::Timestamp::now().as_second();
	let max_time_period = now - (60 * 3);

	if timestamp < 0 || timestamp > now {
		timestamp = 0;
	}
	if timestamp > 0 && timestamp < max_time_period {
		timestamp = max_time_period;
	}

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	let clicker = sqlx::query_as::<_, ClickersRow>("SELECT id, can_mod, name FROM clickers WHERE cookie = $1")
		.bind(&cookie)
		.fetch_optional(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
		.map_or_else(
			|| {
				Err((
					StatusCode::UNAUTHORIZED,
					"no clicker exists with that cookie".to_owned(),
				))
			},
			Ok,
		)?;

	let clickers: Vec<(i64, String, f64, f64, i64, bool)> = sqlx::query_as(
		"
		SELECT
			j.board,
			(CASE
					$1>0
				OR  (j.board!=1 AND c.okay_name>-2)
				OR  (c.okay_name>0 AND c.cheater=0)
				OR  c.id=$2
				WHEN 1
				THEN c.name
				ELSE '???'
			END),
			c.cookies_per_second,
			c.total_cookies,
			c.id,
			(c.okay_name>0 AND c.cheater=0)
		FROM joinedboards j
		JOIN clickers c ON c.id = j.clicker
		JOIN boards b ON b.id = j.board
		WHERE j.board IN (SELECT board FROM joinedboards WHERE clicker = $2)
			  AND ($3 = 0 OR c.last_updated>=$3 OR b.last_updated>=$3)
		ORDER BY
			j.board ASC,
			(j.board=1 AND ((c.okay_name>0 AND c.cheater=0) OR c.id=$2)) DESC,
			c.cookies_per_second DESC,
			c.total_cookies DESC,
			c.id ASC
		",
	)
	.bind(clicker.can_mod)
	.bind(clicker.id)
	.bind(timestamp)
	.fetch_all(&mut *db)
	.await
	.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	// TODO: could query this in parallel in another task...
	let boards: Vec<(i64, String, String, bool)> = sqlx::query_as("SELECT b.id, b.name, (CASE b.owner=$1 WHEN 1 THEN b.cookie ELSE '' END), (b.last_updated>=$2) FROM joinedboards j JOIN boards b ON j.board = b.id WHERE j.clicker = $1 ORDER BY j.board ASC").bind(clicker.id).bind(timestamp).fetch_all(&mut *db).await
	.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	let j = serde_json::json!({
		"boardinfo": boards,
		"boardvalues": clickers,
		"you": clicker.id,
		"can_mod": clicker.can_mod,
		"unsafe_my_name": clicker.name.unwrap_or_default(),
		"timestamp": now
	});

	// TODO: resp.headers["Cache-Control"] = "no-store"

	Ok(j.to_string())
}

async fn leaderboard_leave(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let boardid = get_leaderboard_id(&headers)?;

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	let clickerid: i64 = sqlx::query_scalar("SELECT id FROM clickers WHERE cookie = $1")
		.bind(&cookie)
		.fetch_optional(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
		.map_or_else(
			|| {
				Err((
					StatusCode::UNAUTHORIZED,
					"no clicker exists with that cookie".to_owned(),
				))
			},
			Ok,
		)?;

	let ownerid: i64 = sqlx::query_scalar("SELECT owner FROM boards WHERE id = $1")
		.bind(boardid)
		.fetch_one(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	if clickerid == ownerid {
		sqlx::raw_sql(AssertSqlSafe(format!(
			"
			UPDATE clickers
			SET last_updated=unixepoch()
			WHERE clickers.id IN (SELECT clicker FROM joinedboards WHERE board = {boardid});
			DELETE FROM joinedboards WHERE board = {boardid};
			DELETE FROM boards WHERE id = {boardid};
			"
		)))
		.execute(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;
	} else {
		sqlx::raw_sql(AssertSqlSafe(format!(
			"
			DELETE FROM joinedboards WHERE board = {boardid} AND clicker = {clickerid};
			UPDATE clickers SET last_updated=unixepoch() WHERE id = {clickerid};
			UPDATE boards SET last_updated=unixepoch() WHERE id = {boardid};
			"
		)))
		.execute(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;
		/*
		# IDK if this below even works:
		#cur.execute("""
		#    DELETE FROM joinedboards
		#    LEFT JOIN clickers c ON c.cookie = ?
		#    WHERE board = ? AND clicker = c.id
		#""", (cookie, boardid))
		*/
	}

	Ok("left".to_owned())
}

async fn leaderboard_join(
	headers: HeaderMap,
	State(state): State<Arc<AxumState>>,
) -> Result<String, (StatusCode, String)> {
	check_for_bad_workshop_id(&headers)?;
	let cookie = get_cookie(&headers)?;
	let boardcookie = get_header(&headers, "X-My-Leaderboard-Cookie")?;
	if boardcookie.len() != 32 {
		return Err((StatusCode::BAD_REQUEST, "boardcookie len != 32".to_owned()));
	}

	let mut db = state.pool.acquire().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			"failed to acquire sql connection".to_owned(),
		)
	})?;

	sqlx::query("UPDATE clickers SET last_updated=unixepoch() WHERE cookie = $1")
		.bind(&cookie)
		.execute(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	sqlx::query("UPDATE boards SET last_updated=unixepoch() WHERE cookie = $1")
		.bind(&boardcookie)
		.execute(&mut *db)
		.await
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?;

	let rows_affected = sqlx::query(
		"
		INSERT INTO joinedboards (clicker, board)
		SELECT clickers.id as clicker, boards.id as board
		FROM clickers
		JOIN boards on boards.cookie = $1
		WHERE clickers.cookie = $2
		",
	)
	.bind(boardcookie)
	.bind(cookie)
	.execute(&mut *db)
	.await
	.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "sql query failed".to_owned()))?
	.rows_affected();

	if rows_affected > 0 {
		Ok("joined".to_owned())
	} else {
		Err((StatusCode::INTERNAL_SERVER_ERROR, "???".to_owned()))
	}
}
