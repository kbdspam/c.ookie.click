CREATE TABLE IF NOT EXISTS clickers (
	  id INTEGER PRIMARY KEY
	, okay_name INT NOT NULL DEFAULT 0
	, cheater INT NOT NULL DEFAULT 0
	, can_mod INT NOT NULL DEFAULT 0
	, last_updated INT NOT NULL DEFAULT 0
	, total_cookies REAL NOT NULL DEFAULT 0
	, cookies_per_second REAL NOT NULL DEFAULT 0
	, cookie TEXT NOT NULL
	, name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS boards (
	  id INTEGER PRIMARY KEY
	, owner INT NOT NULL
	, only_owner_cookie INT NOT NULL
	, last_updated INT NOT NULL DEFAULT 0
	, cookie TEXT NOT NULL
	, name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS joinedboards (
	clicker INT NOT NULL
	, board INT NOT NULL
	, UNIQUE(clicker, board) ON CONFLICT IGNORE
);

CREATE INDEX IF NOT EXISTS cookie_clickers ON clickers(cookie);
CREATE INDEX IF NOT EXISTS boards_cookie ON boards(cookie);
CREATE INDEX IF NOT EXISTS cookie_joinedboards ON joinedboards(clicker);
