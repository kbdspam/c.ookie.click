#!/bin/bash
rsync -trvzL --delete \
	fly@merry-sex:/home/fly/cookie/* web/
	#root@bakker:/web2/* web/
#	admin@ec2:/web/* web/

#rsync -trvzL admin@ec2:/web/cookiepy/leaderboard.db web/leaderboard.db
