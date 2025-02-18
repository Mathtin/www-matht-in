#!/bin/sh

SCRIPT_DIR=`dirname "$0"`
LOG_HANDLER="$SCRIPT_DIR/log-handler.sh"

wasm-pack "$@" 2>&1 | sh "$LOG_HANDLER" "wasm-pack"
