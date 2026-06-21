#!/bin/sh

cutechess-cli \
  -engine cmd=./releases/v0.9.0 proto=uci \
  -engine cmd=./releases/v0.8.0 proto=uci \
  -each tc=40/10 \
  -games 277 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout results/v9-v8.pgn \
  -openings file=positions/balanced.epd format=epd
