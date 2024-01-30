#!/bin/bash
rsync -trvzL --delete \
	--exclude cookiepy/venv \
	--exclude cookiepy/__pycache__ \
	--exclude wspollpy/venv \
	--exclude wspollpy/__pycache__ \
	--exclude c.ookie.click/er/topeka \
	boa@bakker:/web/* web2/
#	admin@ec2:/web/* web/

#rsync -trvzL admin@ec2:/web/cookiepy/leaderboard.db web/leaderboard.db
