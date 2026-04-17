
def make_db() -> None:
    db = sqlite3.connect("../data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        CREATE TABLE clickers (id INTEGER PRIMARY KEY, okay_name INT NOT NULL DEFAULT 0, cheater INT NOT NULL DEFAULT 0, can_mod INT NOT NULL DEFAULT 0, last_updated INT NOT NULL DEFAULT 0, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
        CREATE TABLE boards (id INTEGER PRIMARY KEY, owner INT NOT NULL, only_owner_cookie INT NOT NULL, last_updated INT NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
        CREATE TABLE joinedboards (clicker INT NOT NULL, board INT NOT NULL, UNIQUE(clicker, board) ON CONFLICT IGNORE);

        CREATE INDEX cookie_clickers ON clickers(cookie);
        CREATE INDEX boards_cookie ON boards(cookie);
        CREATE INDEX cookie_joinedboards ON joinedboards(clicker);
    """)
    db.commit()

def migrate_db_000() -> None:
    db = sqlite3.connect("../data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        DROP INDEX cookie_clickers;

        CREATE TABLE temp_clickers (id INTEGER PRIMARY KEY, okay_name INT NOT NULL DEFAULT 0, can_mod INT NOT NULL DEFAULT 0, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
        INSERT INTO temp_clickers (id,name,cookie,total_cookies,cookies_per_second) SELECT * FROM clickers;
        DROP TABLE clickers;
        ALTER TABLE temp_clickers RENAME TO clickers;
        CREATE INDEX cookie_clickers ON clickers(cookie);

        UPDATE clickers SET can_mod=1 WHERE id=1 OR id=104;
        UPDATE clickers SET okay_name=1 WHERE name != "1488";

        CREATE TABLE temp_boards (id INTEGER PRIMARY KEY, owner INT NOT NULL, only_owner_cookie INT NOT NULL, cookie TEXT NOT NULL, name TEXT NOT NULL);
        INSERT INTO temp_boards (id,name,owner,cookie,only_owner_cookie) SELECT * FROM boards;
        DROP TABLE boards;
        ALTER TABLE temp_boards RENAME TO boards;
        CREATE INDEX boards_cookie ON boards(cookie);
    """)
    db.commit()
    db.execute("VACUUM;")
    db.commit()

def migrate_db_001() -> None:
    db = sqlite3.connect("../data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        DROP INDEX cookie_clickers;

        CREATE TABLE temp_clickers (id INTEGER PRIMARY KEY, okay_name INT NOT NULL DEFAULT 0, cheater INT NOT NULL DEFAULT 0, can_mod INT NOT NULL DEFAULT 0, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
        INSERT INTO temp_clickers (id,okay_name,can_mod,total_cookies,cookies_per_second,cookie,name) SELECT * FROM clickers;
        UPDATE temp_clickers SET cheater=1, okay_name=0 WHERE okay_name=-2;
        DROP TABLE clickers;
        ALTER TABLE temp_clickers RENAME TO clickers;
        CREATE INDEX cookie_clickers ON clickers(cookie);

        DROP INDEX cookie_joinedboards;
        CREATE TABLE temp_joinedboards (clicker INT NOT NULL, board INT NOT NULL, UNIQUE(clicker, board) ON CONFLICT IGNORE);
        INSERT INTO temp_joinedboards (clicker, board) SELECT * FROM joinedboards;
        DROP TABLE joinedboards;
        ALTER TABLE temp_joinedboards RENAME TO joinedboards;
    """)
    db.commit()
    db.execute("VACUUM;")
    db.commit()

def migrate_db_002() -> None:
    db = sqlite3.connect("../data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        DROP INDEX cookie_clickers;

        CREATE TABLE temp_clickers (id INTEGER PRIMARY KEY, okay_name INT NOT NULL DEFAULT 0, cheater INT NOT NULL DEFAULT 0, can_mod INT NOT NULL DEFAULT 0, last_updated INT NOT NULL DEFAULT 0, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
        INSERT INTO temp_clickers (id,okay_name,cheater,can_mod,total_cookies,cookies_per_second,cookie,name) SELECT id,okay_name,cheater,can_mod,total_cookies,cookies_per_second,cookie,name FROM clickers;
        DROP TABLE clickers;
        ALTER TABLE temp_clickers RENAME TO clickers;
        CREATE INDEX cookie_clickers ON clickers(cookie);
    """)
    db.commit()
    db.execute("VACUUM;")
    db.commit()

def migrate_db_003() -> None:
    db = sqlite3.connect("../data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        DROP INDEX boards_cookie;

        CREATE TABLE temp_boards (id INTEGER PRIMARY KEY, owner INT NOT NULL, only_owner_cookie INT NOT NULL, last_updated INT NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);

        INSERT INTO temp_boards (id,owner,only_owner_cookie,cookie,name) SELECT id,owner,only_owner_cookie,cookie,name FROM boards;
        DROP TABLE boards;
        ALTER TABLE temp_boards RENAME TO boards;
        CREATE INDEX boards_cookie ON boards(cookie);
    """)
    db.commit()
    db.execute("VACUUM;")
    db.commit()

def insert_lots_of_fake_people(targetboard: int) -> None:
    import random
    db = sqlite3.connect("../data/leaderboard.db")
    cur = db.cursor()
    for i in range(100):
        cur.execute("INSERT INTO clickers(name, cookie, total_cookies, cookies_per_second) VALUES (?,?,?,?);", ("faketest"+str(i), randcookie(), random.uniform(1, 10)*1000000,random.uniform(1, 10)*1000000))
        cur.execute("INSERT INTO joinedboards(clicker, board) VALUES (?,?);", (cur.lastrowid, targetboard))
    db.commit()

"""
notes for migrations:
vps:   cp leaderboard.db main.py backup ; sudo systemctl stop cookiepy
home:  ./send.sh  # send main.py
home:  ./recv.sh  # receive database
home:  # edit main.py to run migration code
home:  python main.py  # run migration
home:  # edit main.py to disable migration code
home:  # edit send.sh to send database
home:  ./send.sh  # send database
vps:   sudo systemctl start cookiepy
home:  # edit send.sh to remove sending database
"""

if __name__ == '__main__':
    #make_db()
    #migrate_db_000()
    #migrate_db_001()
    #migrate_db_002()
    #migrate_db_003()
    #insert_lots_of_fake_people(7)
    app.run(host='0.0.0.0', port=12345)
