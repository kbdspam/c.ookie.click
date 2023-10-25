#!/bin/bash
rsync -trvz --delete --exclude cookiepy/venv --exclude cookiepy/__pycache__ --exclude c.ookie.click/er/topeka --exclude cookiepy/leaderboard.db web/* admin@ec2:/web
