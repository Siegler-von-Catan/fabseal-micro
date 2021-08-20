#!/usr/bin/env bash

set -euo pipefail
set -x

host='127.0.0.1:8080'
ep="http://$host/api/v1"
in_file='test.png'
out_file='result.stl'

curl \
    -v \
    --fail-early \
    -b cookies.txt -c cookies.txt \
    -X POST \
    "$ep/create/new" \
    --next \
    -b cookies.txt -c cookies.txt \
    -F "=@$in_file" \
    "$ep/create/upload" \
    --next \
    -b cookies.txt -c cookies.txt \
    -X POST \
    "$ep/create/start"

sleep 30

curl \
    -v \
    -b cookies.txt -c cookies.txt \
    --output "$out_file" \
    -H "Accept: model/stl" \
    "$ep/create/result?type=model" \
