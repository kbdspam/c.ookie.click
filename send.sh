#!/bin/bash
rsync -trvz --delete \
	--exclude cookiepy/venv \
	--exclude cookiepy/__pycache__ \
	--exclude wspollpy/venv \
	--exclude wspollpy/__pycache__ \
	--exclude cookiepy/disabled_registering \
	--exclude cookiepy/disabled_leaderboard_create \
	--exclude c.ookie.click/er/topeka \
	--exclude cookiepy/leaderboard.db \
	--exclude backup \
	web/* boa@bakker:/web
	#web/* admin@ec2:/web
#--exclude cookiepy/leaderboard.db \
