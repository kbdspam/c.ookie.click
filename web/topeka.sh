#!/bin/bash
# crontab -e  with   */2 * * * *  /web/topeka.sh
set -euxo pipefail
TEMP=$(units "tempC($(curl "https://api.weather.gov/stations/KTOP/observations/latest" | jq ."properties.temperature.value"))" "tempF" | xargs)
echo -n "$TEMP" > /web/c.ookie.click/er/topeka
