#!/bin/bash
set -euxo pipefail
python3 -m venv venv
source venv/bin/activate
pip install gunicorn flask
#deactivate
