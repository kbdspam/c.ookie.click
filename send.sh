#!/bin/bash
rsync -trvz --delete \
	--exclude cookiepy/disabled_registering \
	--exclude cookiepy/disabled_leaderboard_create \
	--exclude c.ookie.click/er/topeka \
	--exclude data/leaderboard.db \
	--exclude data/backup \
	--exclude logs \
	web/* debian@cursor-party-0:/home/debian/cookie
	#web/* boa@bakker:/web2
	#web/* admin@ec2:/web
#--exclude cookiepy/leaderboard.db \
