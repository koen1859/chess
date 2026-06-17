#!/bin/sh

cutechess-cli \
  -engine cmd=./releases/v0.5.0 proto=uci \
  -engine cmd=./releases/v0.4.0 proto=uci \
  -each tc=40/10 \
  -games 277 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout results/v5-v4.pgn \
  -openings file=positions/balanced.epd format=epd
