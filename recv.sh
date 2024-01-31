#!/bin/bash
rsync -trvzL --delete \
	root@bakker:/web2/* web/
#	admin@ec2:/web/* web/

#rsync -trvzL admin@ec2:/web/cookiepy/leaderboard.db web/leaderboard.db
