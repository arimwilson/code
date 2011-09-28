package scrabble_test

import ("testing";
        "scrabble"; "util")

func TestBlankScore(t *testing.T) {
  if scrabble.BlankScore(10, 5, '-') != 5 {
    t.Fail()
  }
  if scrabble.BlankScore(10, 3, '1') != 4 {
    t.Fail()
  }
  if scrabble.BlankScore(10, 2, '3') != 3 {
    t.Fail()
  }
}

func TestCanFollow(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  if !scrabble.CanFollow(dict, "ab", map[byte] int {byte('r'): 1}) {
    t.Fail()
  } else if scrabble.CanFollow(dict, "ab", map[byte] int {byte('c'): 1}) {
    t.Fail()
  }
}

