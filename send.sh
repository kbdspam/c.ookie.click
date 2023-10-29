#!/bin/bash
rsync -trvz --delete \
	--exclude cookiepy/venv \
	--exclude cookiepy/__pycache__ \
	--exclude cookiepy/disabled_registering \
	--exclude cookiepy/disabled_leaderboard_create \
	--exclude c.ookie.click/er/topeka \
	--exclude cookiepy/leaderboard.db \
	--exclude backup \
	web/* admin@ec2:/web
