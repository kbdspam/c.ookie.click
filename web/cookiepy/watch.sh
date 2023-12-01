#!/bin/sh
watch -n30 "sqlite3 leaderboard.db \"select * from clickers where okay_name=0;\""
