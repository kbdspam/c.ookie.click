#!/bin/bash
rsync -trvzL --delete admin@ec2:/web/* web/
