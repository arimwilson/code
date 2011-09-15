#!/bin/bash

gd . -o scrabble && ./scrabble -w twl.txt -b sample.txt -t "SAEDBQH"
gd . -c
