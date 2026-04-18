#!/bin/bash
# crontab -e  with   */5 * * * *  /web/cookiepy/backupdb.sh
set -euo pipefail
mkdir -p /data/backup
backupto="/data/backup/leaderboard_$(date +'%F_%T').db"
sqlite3 /data/leaderboard.db "VACUUM INTO '$backupto'"
gzip "$backupto"
