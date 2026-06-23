#!/bin/sh

cutechess-cli \
  -engine cmd=./releases/v0.12.0 proto=uci \
  -engine cmd=stockfish proto=uci \
  -each tc=40/10 \
  -games 277 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout results/v12-stockfish.pgn \
  -openings file=positions/balanced.epd format=epd
