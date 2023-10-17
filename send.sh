#!/bin/bash
rsync -trvz --delete web/* admin@ec2:/web
