#!/bin/sh

SCRIPTPATH="$(cd "$(dirname "$0")" && pwd)"

wasm-pack build "$SCRIPTPATH/rust" --target web --out-dir "$SCRIPTPATH/../js/src/wasm"