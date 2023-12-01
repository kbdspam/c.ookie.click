#!/bin/bash
# crontab -e  with   */5 * * * *  /web/cookiepy/backupdb.sh
set -euxo pipefail
mkdir -p /web/cookiepy/backup
backupto="/web/cookiepy/backup/leaderboard_$(date +'%F_%T').db"
sqlite3 /web/cookiepy/leaderboard.db "VACUUM INTO '$backupto'"
gzip "$backupto"
