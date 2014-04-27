#!/bin/bash
find $1 -regex '.*cc$\|.*c$\|.*h$\|.*py$\|.*go$\|.*js$\|.*java$\|.*hs\|.*sh$\|.*html$' -not -path '*/\.*' -type f | xargs wc -l | sort -b -g -r
