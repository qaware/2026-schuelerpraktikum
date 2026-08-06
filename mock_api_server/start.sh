#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"

PORT="${PORT:-8000}"

if [ ! -d "venv" ]; then
    echo "Creating virtual environment in $DIR/venv..."
    python3 -m venv venv
    echo "Installing requirements..."
    ./venv/bin/pip install -q -r requirements.txt
fi

echo "=========================================================="
echo " Starting Satellite Mock REST API Server on port $PORT"
echo " Interactive Swagger API Docs: http://localhost:$PORT/docs"
echo " Endpoints defined by api_spezifikation.md"
echo "=========================================================="

./venv/bin/python main.py
