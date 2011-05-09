#!/bin/bash

6g moves.go && 6g sortwith.go && 6g trie.go && 6g util.go && 6g scrabble.go && 6l scrabble.6 && ./6.out -w twl.txt -b sample.txt -t "saedbqh"
rm 6.out trie.6 moves.6 sortwith.6 scrabble.6
