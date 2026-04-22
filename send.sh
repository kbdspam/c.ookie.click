#!/bin/bash
rsync -trvz --delete \
	--exclude cookiepy/disabled_registering \
	--exclude cookiepy/disabled_leaderboard_create \
	--exclude c.ookie.click/er/topeka \
	--exclude data/leaderboard.db \
	--exclude data/backup \
	--exclude logs \
	web/* root@atlas:/home/atlas/cookie
#--exclude cookiepy/leaderboard.db \
