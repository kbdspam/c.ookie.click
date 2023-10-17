#!/bin/bash
set -euxo pipefail
TEMP=$(units "tempC($(curl "https://api.weather.gov/stations/KTOP/observations/latest" | jq ."properties.temperature.value"))" "tempF" | xargs)
echo -n "$TEMP" > /web/c.ookie.click/er/topeka
