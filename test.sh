#!/bin/sh

cutechess-cli \
  -engine cmd=./target/release/engine-uci proto=uci depth=7 \
  -engine cmd=stockfish proto=uci depth=3 \
  -each tc=40/10 \
  -games 16 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout games.pgn
