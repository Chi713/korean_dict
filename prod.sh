#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
bash -c  'cargo watch -- cargo run --release -- --port 8080 --static-dir ../htmx-frontend')

