from flask import Flask,g,abort,request,jsonify
import sqlite3
import secrets
import datetime

# TODO: rate-limiting

app = Flask(__name__)

def get_db(autocommit): #idk about autocommit stuff rn
    db = getattr(g, '_database', None)
    if db is None:
        db = g._database = sqlite3.connect("leaderboard.db", autocommit=autocommit)
    return db

def isnan(num):
    return num != num

@app.teardown_appcontext
def close_connection(exception):
    db = getattr(g, '_database', None)
    if db is not None:
        db.close()

@app.route('/er/leaderboard/register', methods=['POST'])
def leaderboard_register():
    name = request.form.get("name", "")
    if len(name) < 1 or len(name) > 32 or len(name.encode('utf-8')) > 32:
        return "name too big or too small", 400
    cookie = ''.join(secrets.choices("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890", k=32))
    cur = get_db(True).cursor()
    cur.execute("INSERT INTO clickers(name, cookie) VALUES (?,?,0,0);", (name, cookie))
    return cookie, 200

@app.route('/er/leaderboard/create', methods=['POST'])
def leaderboard_create():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    name = request.headers.get('X-My-Leaderboard-Name', '')
    if len(name) < 1 or len(name) > 32 or len(name.encode('utf-8')) > 32:
        return "name too big or too small", 400
    boardcookie = ''.join(secrets.choices("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890", k=32))
    cur = get_db(True).cursor()
    cid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()[0]
    if 5 <= cur.execute("SELECT COUNT(*) FROM joinedboards WHERE clicker = ?", (cid,)).fetchone()[0]:
        return "in too many boards", 400
    cur.executescript("""
        INSERT INTO boards(name, owner, cookie) VALUES (?,?,?);
        INSERT INTO joinedboards(owner, board) VALUES (?, last_insert_rowid());
    """, (name, cid, boardcookie,  cid))
    return "", 200

@app.route('/er/leaderboard/updateme', methods=['POST'])
def leaderboard_updateme():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    data = request.headers.get('X-My-Update-Data', '-1|-1').split('|')
    total_cookies = float(data[0])
    cookies_per_second = float(data[1])
    if isnan(total_cookies) or total_cookies < 0 or isnan(cookies_per_second) or cookies_per_second < 0:
        raise Exception("nan or less than 0")
    cur = get_db(True).cursor()
    cur.execute("UPDATE clickers SET total_cookies = ?, cookies_per_second = ? WHERE cookie = ?", (total_cookies, cookies_per_second, cookie))
    if cur.rowcount > 0:
        return "updated", 200
    else:
        return "???", 500

@app.route('/er/leaderboard/query')
def leaderboard_query():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "", 401
    cid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()[0]
    res = cur.execute("""
        SELECT j.board, c.name, c.total_cookies, c.cookies_per_second
        FROM joinedboards j
        JOIN clickers c ON c.id = j.clicker
        WHERE j.board IN (SELECT board FROM joinedboards WHERE clicker = ?)
        ORDER BY j.board ASC, c.cookies_per_second DESC, c.total_cookies DESC, c.id ASC
    """, (cid,)).fetchall()
    boards = cur.execute("SELECT b.id, b.name FROM joinedboards j JOIN boards b ON j.board = b.id WHERE j.clicker = ?", (cid,)).fetchall()
    return jsonify(crud=res,boards=boards)

@app.route('/er/leaderboard/leave')
def leaderboard_leave():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db(True).cursor()
    # lol... atomicity and transactions? never heard of them...
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()[0]
    ownerid = cur.execute("SELECT owner FROM boards WHERE id = ?", (boardid,)).fetchone()[0]
    if clickerid == ownerid:
        cur.executescript("""
            BEGIN;
            DELETE FROM joinedboards WHERE boardid = ?;
            DELETE FROM boards WHERE id = ?;
            COMMIT;
        """, (boardid, boardid))
    else:
        cur.execute("DELETE FROM joinedboards WHERE board = ? AND clicker = ?", (boardid, clickerid))
        # IDK if this below even works:
        #cur.execute("""
        #    DELETE FROM joinedboards 
        #    LEFT JOIN clickers c ON c.cookie = ?
        #    WHERE board = ? AND clicker = c.id
        #""", (cookie, boardid))
    return "left", 200

@app.route('/er/leaderboard/join', methods=['POST'])
def leaderboard_join():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardcookie = request.headers.get('X-My-Leaderboard-Cookie', '')
    if len(boardcookie) != 32:
        return "b", 401
    cur = get_db(True).cursor()
    cur.execute("""
        INSERT INTO joinedboards (cid, bid)
        SELECT clickers.id as cid, boards.id as bid
        FROM clickers WHERE cookie = ?
        JOIN boards on boards.cookie = ?
    """, (cookie, boardcookie))
    if cur.rowcount > 0:
        return "joined", 200
    else:
        return "???", 500

def make_db():
    db = sqlite3.connect("leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        CREATE TABLE clickers (id INTEGER PRIMARY KEY, name TEXT NOT NULL, cookie TEXT NOT NULL, total_cookies REAL NOT NULL, cookies_per_second REAL NOT NULL);
        CREATE TABLE boards (id INTEGER PRIMARY KEY, name TEXT NOT NULL, owner INT NOT NULL, cookie TEXT NOT NULL, only_owner_cookie INT NOT NULL);
        CREATE TABLE joinedboards (clicker INT NOT NULL, board INT NOT NULL);
        
        CREATE INDEX cookie_clickers ON clickers(cookie);
        CREATE INDEX cookie_joinedboards ON joinedboards(clicker);
    """)
    db.commit()

if __name__ == '__main__':
    make_db()
    #app.run(host='127.0.0.1', port=12345)
