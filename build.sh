#!/bin/sh

SCRIPTPATH=$(dirname "$SCRIPT")

wasm-pack build "$SCRIPTPATH/wasm" --target web --out-dir "$SCRIPTPATH/../src/wasm"
