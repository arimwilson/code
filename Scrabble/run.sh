#!/bin/bash
#
# Copyright 2011 Google Inc. All Rights Reserved.
# Author: ariw@google.com (Ari Wilson)

6g trie.go && 6g moves.go && 6g sortwith.go && 6g scrabble.go && 6l scrabble.6 && ./6.out -w twl.txt -b sample.txt -t "saedbqh"
