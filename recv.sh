#!/bin/bash
rsync -trvzL --delete --exclude cookiepy/venv --exclude c.ookie.click/er/topeka admin@ec2:/web/* web/
