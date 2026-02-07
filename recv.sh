#!/bin/bash
rsync -trvzL --delete \
	root@atlas:/home/atlas/cookie/* web/

#rsync -trvzL admin@ec2:/web/cookiepy/leaderboard.db web/leaderboard.db
