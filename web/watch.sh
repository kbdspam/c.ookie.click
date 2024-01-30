#!/bin/sh
watch -n30 "sqlite3 data/leaderboard.db \"select * from clickers where okay_name=0;\""
