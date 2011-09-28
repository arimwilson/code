package trie_test

import ("testing";
        "util")

func TestInsertAndRetrieve(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  if !dict.Find("abba") { t.Fail() }
  if !dict.Find("textual") { t.Fail() }
}

func TestNonExistent(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  if dict.Find("ari") { t.Fail() }
  if dict.Find("xylophone") { t.Fail() }
}

func TestFollowing(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  following := dict.Following("ab")
  if len(following) != 2 { t.Fail() }
  if following[0] != 'b' { t.Fail() }
  if following[1] != 'r' { t.Fail() }
}

