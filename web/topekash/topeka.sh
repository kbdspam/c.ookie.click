#!/bin/bash
# crontab -e  with   */2 * * * *  /web/topeka.sh
set -euo pipefail
TEMP=$(units "tempC($(curl -s "https://api.weather.gov/stations/KTOP/observations/latest" | jq ."properties.temperature.value"))" "tempF" | xargs)
echo -n "$TEMP" > /app/topeka
