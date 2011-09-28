package trie_test

import ("testing";
        "util")

func TestInsertAndRetrieve(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  if !dict.Find("ABBA") { t.Fail() }
  if !dict.Find("TEXTUAL") { t.Fail() }
}

func TestNonExistent(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  if dict.Find("ARI") { t.Fail() }
  if dict.Find("XYLOPHONE") { t.Fail() }
}

func TestFollowing(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  following := dict.Following("AB")
  if len(following) != 2 { t.Fail() }
  if following[0] != 'B' { t.Fail() }
  if following[1] != 'R' { t.Fail() }
}

