from flask import Flask,g,abort,request,jsonify,Response
import sqlite3
import secrets
import math
import os.path
import re
import time

# TODO: rate-limiting

# https://docs.astral.sh/ruff/
# uvx ruff check

app = Flask(__name__)

def get_db() -> sqlite3.Connection:
    db = getattr(g, '_database', None)
    if db is None:
        db = g._database = sqlite3.connect("/data/leaderboard.db")
    return db

def badnum(num: float) -> bool:
    return num != num or math.isinf(num)
def isOkayName(s: str) -> bool:
    b = s.strip().encode("utf-8")
    if len(b) < 1 or len(b) > 31:
        return False
    return all(c >= 0x20 for c in b)

def randcookie() -> str:
    return ''.join(secrets.choice("0123456789abcdefABCDEF") for _ in range(32))

def disabled_registering() -> bool:
    return os.path.isfile("/data/disabled_registering")
def disabled_leaderboard_create() -> bool:
    return os.path.isfile("/data/disabled_leaderboard_create")

def bad_workshop_id() -> bool:
    if time.time() > 1724017056: # Sun Aug 18 2024 21:37:36 GMT+0000
        return request.headers.get('X-My-Workshop-ID', '') != "3061304069"
    else:
        return False

@app.teardown_appcontext
def close_connection(exception) -> None:
    db = getattr(g, '_database', None)
    if db is not None:
        db.close()

@app.route('/er/leaderboard/register', methods=['POST'])
def leaderboard_register() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    if disabled_registering():
        return "db broken", 500 # shhh
    #name = request.form.get("name", "") # TODO
    name = request.headers.get('X-My-New-Leaderboard-Name', '').strip()
    if not isOkayName(name):
        return "name too big or too small", 400
    cookie = randcookie()
    cur = get_db().cursor()
    cur.execute("INSERT INTO clickers(name, cookie) VALUES (?,?);", (name, cookie))
    #cur.execute("INSERT INTO joinedboards(clicker, board) VALUES (?,1);", (cur.last_insert_rowid,))
    get_db().commit()
    return cookie, 200

@app.route('/er/leaderboard/changemyname', methods=['POST'])
def leaderboard_changemyname() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    if disabled_registering():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    name = request.headers.get('X-My-New-Leaderboard-Name', '').strip()
    if not isOkayName(name):
        return "name too big or too small", 400
    cur = get_db().cursor()
    # special handling so I don't have to continue okay_name=1 'ing a person
    if re.fullmatch(r'i am #\d{3} on global', name) is not None:
        _ = cur.execute("""
            UPDATE clickers SET
            name=?,
            okay_name=(CASE okay_name WHEN -2 THEN -2 ELSE 1 END)
            WHERE cookie = ?""", (name,cookie))
    else:
        _ = cur.execute("""
            UPDATE clickers SET
            name=?,
            last_updated=unixepoch(),
            okay_name=(CASE okay_name WHEN -2 THEN -2 ELSE 0 END)
            WHERE cookie = ?""", (name,cookie))
    get_db().commit()
    if cur.rowcount > 0:
        # TODO: Ping me on discord to whitelist...
        return "changed", 200
    else:
        return "no?", 400

@app.route('/er/leaderboard/create', methods=['POST'])
def leaderboard_create() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    if disabled_leaderboard_create():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    name = request.headers.get('X-My-New-Leaderboard-Name', '').strip()
    if not isOkayName(name):
        return "name too big or too small", 400
    boardcookie = randcookie()
    cur = get_db().cursor()
    cid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if cid is None:
        abort(403)
    cid = cid[0]
    if 5 <= cur.execute("SELECT COUNT(*) FROM joinedboards WHERE clicker = ?", (cid,)).fetchone()[0]:
        return "in too many boards", 400
    cur.execute("INSERT INTO boards(name, owner, cookie, only_owner_cookie, last_updated) VALUES (?,?,?,0,unixepoch());", (name, cid, boardcookie))
    cur.execute("INSERT INTO joinedboards(clicker, board) VALUES (?, last_insert_rowid());", (cid,))
    get_db().commit()
    return "", 200

@app.route('/er/leaderboard/cycleboardcookie', methods=['POST'])
def leaderboard_cycleboardcookie() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db().cursor()
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if clickerid is None:
        abort(403)
    clickerid = clickerid[0]
    _ = cur.execute("UPDATE boards SET cookie = ?, last_updated=unixepoch() WHERE owner = ? AND id = ?", (randcookie(),clickerid,boardid))
    get_db().commit()
    if cur.rowcount > 0:
        return "cycled", 200
    else:
        return "no?", 400

@app.route('/er/leaderboard/changeboardname', methods=['POST'])
def leaderboard_changeboardname() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    name = request.headers.get('X-My-New-Leaderboard-Name', '').strip()
    if not isOkayName(name):
        return "name too big or too small", 400
    cur = get_db().cursor()
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if clickerid is None:
        abort(403)
    clickerid = clickerid[0]
    _ = cur.execute("UPDATE boards SET name = ?, last_updated=unixepoch() WHERE owner = ? AND id = ?", (name,clickerid,boardid))
    get_db().commit()
    if cur.rowcount > 0:
        return "changed", 200
    else:
        return "no?", 400

@app.route('/er/leaderboard/kick', methods=['POST'])
def leaderboard_kick() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    enemy = int(request.headers.get('X-My-Enemy-ID', ''))
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db().cursor()
    res = cur.execute("SELECT id, can_mod FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if res is None:
        abort(403)
    clickerid = res[0]
    can_mod = res[1]
    if enemy == clickerid:
        return "no...", 400
    can_kick = False
    if boardid == 1:
        can_kick = can_mod >= 2
    else:
        cur.execute("UPDATE boards SET cookie = ? WHERE owner = ? AND id = ?", (randcookie(),clickerid,boardid))
        can_kick = cur.rowcount > 0
    if can_kick:
        cur.execute("DELETE FROM joinedboards WHERE clicker = ? AND board = ?", (enemy,boardid))
        if cur.rowcount > 0:
            cur.executescript("""
                UPDATE clickers SET last_updated=unixepoch() WHERE id={enemy};
                UPDATE boards SET last_updated=unixepoch() WHERE id={boardid};
            """)
            get_db().commit()
            return "kicked", 200
        else:
            return "not in board?", 400
    else:
        return "no dice", 400

@app.route('/er/leaderboard/updateme', methods=['POST'])
def leaderboard_updateme() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    data = request.headers.get('X-My-Update-Data', '-1|-1').split('|')
    total_cookies = float(data[0])
    cookies_per_second = float(data[1])
    cur = get_db().cursor()
    if badnum(total_cookies) or total_cookies < 0 or badnum(cookies_per_second) or cookies_per_second < 0:
        cur.execute("UPDATE clickers SET cheater=1, last_updated=unixepoch() WHERE cookie = ?", (cookie,))
        get_db().commit()
        return "", 500
        #raise Exception("nan or less than 0")
    cur.execute("UPDATE clickers SET last_updated=unixepoch(), total_cookies = ?, cookies_per_second = ? WHERE cookie = ?", (total_cookies, cookies_per_second, cookie))
    get_db().commit()
    if cur.rowcount > 0:
        return "updated", 200
    else:
        return "???", 403

@app.route('/er/leaderboard/query')
def leaderboard_query() -> Response | tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "", 401
    try:
        timestamp = int(request.headers.get('X-My-Timestamp2', '0'))
    except ValueError:
        timestamp = 0
    now = int(time.time())
    max_time_period = now - (60 * 3)
    if timestamp < 0 or timestamp > now:
        timestamp = 0
    if timestamp > 0 and timestamp < max_time_period:
        timestamp = max_time_period
    cur = get_db().cursor()
    res = cur.execute("SELECT id, can_mod, name FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    """
    if res is None:
        cur.execute("INSERT INTO clickers(name, cookie) VALUES (?,?);", ("dev broke db; change name", cookie))
        get_db().commit()
        abort(403)
    """
    if res is None:
        abort(403)
    cid = res[0]
    can_mod = res[1]
    unsafe_my_name = res[2]
    res = cur.execute(f"""
        SELECT
            j.board,
            (CASE
                    {can_mod}>0
                OR  (j.board!=1 AND c.okay_name>-2)
                OR  (c.okay_name>0 AND c.cheater=0)
                OR  c.id={cid}
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
        WHERE j.board IN (SELECT board FROM joinedboards WHERE clicker = {cid})
              AND ({timestamp} = 0 OR c.last_updated>={timestamp} OR b.last_updated>={timestamp})
        ORDER BY
            j.board ASC,
            (j.board=1 AND ((c.okay_name>0 AND c.cheater=0) OR c.id={cid})) DESC,
            c.cookies_per_second DESC,
            c.total_cookies DESC,
            c.id ASC
    """).fetchall()
    boards = cur.execute("SELECT b.id, b.name, (CASE b.owner=? WHEN 1 THEN b.cookie ELSE '' END) FROM joinedboards j JOIN boards b ON j.board = b.id WHERE j.clicker = ? ORDER BY j.board ASC", (cid,cid,)).fetchall()
    resp = jsonify(boardinfo=boards,boardvalues=res,you=cid,can_mod=can_mod,unsafe_my_name=unsafe_my_name,timestamp=now)
    resp.headers["Cache-Control"] = "no-store"
    return resp

@app.route('/er/leaderboard/leave', methods=['POST'])
def leaderboard_leave() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db().cursor()
    # lol... atomicity and transactions? never heard of them...
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if clickerid is None:
        abort(403)
    clickerid = clickerid[0]
    ownerid = cur.execute("SELECT owner FROM boards WHERE id = ?", (boardid,)).fetchone()
    if ownerid is None:
        abort(403)
    ownerid = ownerid[0]
    if clickerid == ownerid:
        cur.execute("""
            UPDATE clickers
            SET last_updated=unixepoch()
            WHERE clickers.id IN (SELECT clicker FROM joinedboards WHERE board = ?)
        """, (boardid,))
        cur.execute("DELETE FROM joinedboards WHERE board = ?;", (boardid,))
        cur.execute("DELETE FROM boards WHERE id = ?;", (boardid,))
    else:
        cur.execute("DELETE FROM joinedboards WHERE board = ? AND clicker = ?", (boardid, clickerid))
        cur.execute("UPDATE clickers SET last_updated=unixepoch() WHERE id = ?", (clickerid,))
        cur.execute("UPDATE boards SET last_updated=unixepoch() WHERE id = ?", (boardid,))
        # IDK if this below even works:
        #cur.execute("""
        #    DELETE FROM joinedboards
        #    LEFT JOIN clickers c ON c.cookie = ?
        #    WHERE board = ? AND clicker = c.id
        #""", (cookie, boardid))

    get_db().commit()
    return "left", 200

@app.route('/er/leaderboard/join', methods=['POST'])
def leaderboard_join() -> tuple[str, int]:
    if bad_workshop_id():
        return "db broken", 500 # shhh
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardcookie = request.headers.get('X-My-Leaderboard-Cookie', '')
    if len(boardcookie) != 32:
        return "b", 401
    cur = get_db().cursor()
    cur.execute("UPDATE clickers SET last_updated=unixepoch() WHERE cookie = ?", (cookie,))
    cur.execute("UPDATE boards SET last_updated=unixepoch() WHERE cookie = ?", (boardcookie,))
    cur.execute("""
        INSERT INTO joinedboards (clicker, board)
        SELECT clickers.id as clicker, boards.id as board
        FROM clickers
        JOIN boards on boards.cookie = ?
        WHERE clickers.cookie = ?
    """, (boardcookie,cookie))
    get_db().commit()
    if cur.rowcount > 0:
        return "joined", 200
    else:
        return "???", 500

def make_db() -> None:
    db = sqlite3.connect("/data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        CREATE TABLE clickers (id INTEGER PRIMARY KEY, okay_name INT NOT NULL DEFAULT 0, cheater INT NOT NULL DEFAULT 0, can_mod INT NOT NULL DEFAULT 0, last_updated INT NOT NULL DEFAULT 0, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
        CREATE TABLE boards (id INTEGER PRIMARY KEY, owner INT NOT NULL, only_owner_cookie INT NOT NULL, cookie TEXT NOT NULL, name TEXT NOT NULL, last_updated INT NOT NULL DEFAULT 0);
        CREATE TABLE joinedboards (clicker INT NOT NULL, board INT NOT NULL, UNIQUE(clicker, board) ON CONFLICT IGNORE);

        CREATE INDEX cookie_clickers ON clickers(cookie);
        CREATE INDEX boards_cookie ON boards(cookie);
        CREATE INDEX cookie_joinedboards ON joinedboards(clicker);
    """)
    db.commit()

def migrate_db_000() -> None:
    db = sqlite3.connect("/data/leaderboard.db")
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
    db = sqlite3.connect("/data/leaderboard.db")
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
    db = sqlite3.connect("/data/leaderboard.db")
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
    db = sqlite3.connect("/data/leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        DROP INDEX boards_cookie;

        CREATE TABLE temp_boards (id INTEGER PRIMARY KEY, owner INT NOT NULL, only_owner_cookie INT NOT NULL, cookie TEXT NOT NULL, name TEXT NOT NULL, last_updated INT NOT NULL DEFAULT 0);

        CREATE TABLE temp_clickers (id INTEGER PRIMARY KEY, okay_name INT NOT NULL DEFAULT 0, cheater INT NOT NULL DEFAULT 0, can_mod INT NOT NULL DEFAULT 0, last_updated INT NOT NULL DEFAULT 0, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0, cookie TEXT NOT NULL, name TEXT NOT NULL);
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
    db = sqlite3.connect("/data/leaderboard.db")
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
