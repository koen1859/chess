#!/bin/sh

cutechess-cli \
  -engine cmd=./releases/v0.6.0 proto=uci \
  -engine cmd=./releases/v0.1.0 proto=uci \
  -each tc=40/10 \
  -games 277 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout results/v6-v1.pgn \
  -openings file=positions/balanced.epd format=epd
