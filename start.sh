#!/bin/sh
cd /home/koenstevens/chess
exec /run/current-system/sw/bin/nix develop --command cargo run --release -- -p 5003
