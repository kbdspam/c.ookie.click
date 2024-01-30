#!/bin/bash
rsync -trvz --delete \
	--exclude cookiepy/venv \
	--exclude cookiepy/__pycache__ \
	--exclude wspollpy/venv \
	--exclude wspollpy/__pycache__ \
	--exclude cookiepy/disabled_registering \
	--exclude cookiepy/disabled_leaderboard_create \
	--exclude c.ookie.click/er/topeka \
	--exclude data/leaderboard.db \
	--exclude data/backup \
	--exclude logs \
	web/* boa@bakker:/web2
	#web/* admin@ec2:/web
#--exclude cookiepy/leaderboard.db \
