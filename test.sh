#!/bin/sh

cutechess-cli \
  -engine cmd=./releases/v0.13.0 proto=uci \
  -engine cmd=./releases/v0.14.0 proto=uci \
  -each tc=40/10 \
  -games 277 \
  -repeat \
  -concurrency $(nproc) \
  -pgnout results/v13-v14.pgn \
  -openings file=positions/balanced.epd format=epd
