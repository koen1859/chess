#!/bin/sh

cutechess-cli \
  -engine cmd=./releases/v0.3.0 proto=uci \
  -engine cmd=./releases/v0.2.0 proto=uci \
  -each tc=40/10 \
  -games 277 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout results/v3-v2.pgn \
  -openings file=positions/balanced.epd format=epd
