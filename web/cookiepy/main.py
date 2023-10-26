from flask import Flask,g,abort,request,jsonify
import sqlite3
import secrets
import datetime

# TODO: rate-limiting

app = Flask(__name__)

def get_db():
    db = getattr(g, '_database', None)
    if db is None:
        db = g._database = sqlite3.connect("leaderboard.db")
    return db

def isnan(num):
    return num != num

def randcookie():
    return ''.join(secrets.choice("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890") for _ in range(32))

@app.teardown_appcontext
def close_connection(exception):
    db = getattr(g, '_database', None)
    if db is not None:
        db.close()

@app.route('/er/leaderboard/register', methods=['POST'])
def leaderboard_register():
    #name = request.form.get("name", "") # TODO
    name = request.headers.get('X-My-New-Leaderboard-Name', '')
    if len(name) < 1 or len(name) > 31 or len(name.encode('utf-8')) > 31:
        return "name too big or too small", 400
    cookie = randcookie()
    cur = get_db().cursor()
    cur.execute("INSERT INTO clickers(name, cookie) VALUES (?,?);", (name, cookie))
    get_db().commit()
    return cookie, 200

@app.route('/er/leaderboard/create', methods=['POST'])
def leaderboard_create():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    name = request.headers.get('X-My-New-Leaderboard-Name', '')
    if len(name) < 1 or len(name) > 31 or len(name.encode('utf-8')) > 31:
        return "name too big or too small", 400
    boardcookie = randcookie()
    cur = get_db().cursor()
    cid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if cid == None: abort(403)
    cid = cid[0]
    if 5 <= cur.execute("SELECT COUNT(*) FROM joinedboards WHERE clicker = ?", (cid,)).fetchone()[0]:
        return "in too many boards", 400
    cur.execute("INSERT INTO boards(name, owner, cookie, only_owner_cookie) VALUES (?,?,?,0);", (name, cid, boardcookie))
    cur.execute("INSERT INTO joinedboards(clicker, board) VALUES (?, last_insert_rowid());", (cid,))
    get_db().commit()
    return "", 200

@app.route('/er/leaderboard/cycleboardcookie', methods=['POST'])
def leaderboard_cycleboardcookie():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db().cursor()
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if clickerid == None: abort(403)
    clickerid = clickerid[0]
    r = cur.execute("UPDATE boards SET cookie = ? WHERE owner = ? AND id = ?", (randcookie(),clickerid,boardid))
    get_db().commit()
    if cur.rowcount > 0:
        return "cycled", 200
    else:
        return "no?", 400

@app.route('/er/leaderboard/kick', methods=['POST'])
def leaderboard_kick():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    enemy = int(request.headers.get('X-My-Enemy-ID', ''))
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db().cursor()
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if clickerid == None: abort(403)
    clickerid = clickerid[0]
    if enemy == clickerid:
        return "no...", 400
    r = cur.execute("UPDATE boards SET cookie = ? WHERE owner = ? AND id = ?", (randcookie(),clickerid,boardid))
    if cur.rowcount > 0:
        r = cur.execute("DELETE FROM joinedboards WHERE clicker = ? AND board = ?", (enemy,boardid))
        if cur.rowcount > 0:
            get_db().commit()
            return "kicked", 200
        else:
            return "not in board?", 400
    else:
        return "???", 400

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
    cur = get_db().cursor()
    cur.execute("UPDATE clickers SET total_cookies = ?, cookies_per_second = ? WHERE cookie = ?", (total_cookies, cookies_per_second, cookie))
    get_db().commit()
    if cur.rowcount > 0:
        return "updated", 200
    else:
        return "???", 403

@app.route('/er/leaderboard/query')
def leaderboard_query():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "", 401
    cur = get_db().cursor()
    cid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if cid == None: abort(403)
    cid = cid[0]
    res = cur.execute("""
        SELECT j.board, c.name, c.cookies_per_second, c.total_cookies, c.id
        FROM joinedboards j
        JOIN clickers c ON c.id = j.clicker
        WHERE j.board IN (SELECT board FROM joinedboards WHERE clicker = ?)
        ORDER BY j.board ASC, c.cookies_per_second DESC, c.total_cookies DESC, c.id ASC
    """, (cid,)).fetchall()
    boards = cur.execute("SELECT b.id, b.name, (CASE b.owner=? WHEN 1 THEN b.cookie ELSE '' END) FROM joinedboards j JOIN boards b ON j.board = b.id WHERE j.clicker = ? ORDER BY j.board ASC", (cid,cid,)).fetchall()
    return jsonify(boardinfo=boards,boardvalues=res,you=cid)

@app.route('/er/leaderboard/leave', methods=['POST'])
def leaderboard_leave():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardid = int(request.headers.get('X-My-Leaderboard-ID', ''))
    cur = get_db().cursor()
    # lol... atomicity and transactions? never heard of them...
    clickerid = cur.execute("SELECT id FROM clickers WHERE cookie = ?", (cookie,)).fetchone()
    if clickerid == None: abort(403)
    clickerid = clickerid[0]
    ownerid = cur.execute("SELECT owner FROM boards WHERE id = ?", (boardid,)).fetchone()
    if ownerid == None: abort(403)
    ownerid = ownerid[0]
    if clickerid == ownerid:
        cur.execute("DELETE FROM joinedboards WHERE board = ?;", (boardid,))
        cur.execute("DELETE FROM boards WHERE id = ?;", (boardid,))
    else:
        cur.execute("DELETE FROM joinedboards WHERE board = ? AND clicker = ?", (boardid, clickerid))
        # IDK if this below even works:
        #cur.execute("""
        #    DELETE FROM joinedboards 
        #    LEFT JOIN clickers c ON c.cookie = ?
        #    WHERE board = ? AND clicker = c.id
        #""", (cookie, boardid))
    get_db().commit()
    return "left", 200

@app.route('/er/leaderboard/join', methods=['POST'])
def leaderboard_join():
    cookie = request.headers.get('X-My-Cookie', '')
    if len(cookie) != 32:
        return "a", 401
    boardcookie = request.headers.get('X-My-Leaderboard-Cookie', '')
    if len(boardcookie) != 32:
        return "b", 401
    cur = get_db().cursor()
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

def make_db():
    db = sqlite3.connect("leaderboard.db")
    cur = db.cursor()
    cur.executescript("""
        CREATE TABLE clickers (id INTEGER PRIMARY KEY, name TEXT NOT NULL, cookie TEXT NOT NULL, total_cookies REAL NOT NULL DEFAULT 0, cookies_per_second REAL NOT NULL DEFAULT 0);
        CREATE TABLE boards (id INTEGER PRIMARY KEY, name TEXT NOT NULL, owner INT NOT NULL, cookie TEXT NOT NULL, only_owner_cookie INT NOT NULL);
        CREATE TABLE joinedboards (clicker INT NOT NULL, board INT NOT NULL);
        
        CREATE INDEX cookie_clickers ON clickers(cookie);
        CREATE INDEX cookie_joinedboards ON joinedboards(clicker);
    """)
    db.commit()

def insert_lots_of_fake_people(targetboard):
    import random
    db = sqlite3.connect("leaderboard.db")
    cur = db.cursor()
    for i in range(100):
        cur.execute("INSERT INTO clickers(name, cookie, total_cookies, cookies_per_second) VALUES (?,?,?,?);", ("faketest"+str(i), randcookie(), random.uniform(1, 10)*1000000,random.uniform(1, 10)*1000000))
        cur.execute("INSERT INTO joinedboards(clicker, board) VALUES (?,?);", (cur.lastrowid, targetboard))
    db.commit()

if __name__ == '__main__':
    #make_db()
    #insert_lots_of_fake_people(7)
    app.run(host='127.0.0.1', port=12345)
