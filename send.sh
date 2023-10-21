#!/bin/bash
rsync -trvz --delete --exclude cookiepy/venv --exclude c.ookie.click/er/topeka web/* admin@ec2:/web
