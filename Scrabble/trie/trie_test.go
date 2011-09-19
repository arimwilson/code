package trie_test

import ("testing";
        "trie")

var strings = []string{
  "abba",
  "abra",
  "existing",
  "textual",
  "later"}

func InsertIntoDictionary() (dict* trie.Trie) {
  dict = trie.New()
  for i := 0; i < len(strings); i++ {
    dict.Insert(strings[i])
  }
  return
}

func TestInsertAndRetrieve(t *testing.T) {
  dict := InsertIntoDictionary()
  for i := 0; i < len(strings); i++ {
    if !dict.Find(strings[i]) {
      t.Errorf("Could not find %s in dict.", strings[i])
    }
  }
  return
}

func TestNonExistent(t *testing.T) {
  dict := InsertIntoDictionary()
  if (dict.Find("ari")) { t.Fail() }
  if (dict.Find("xylophone")) { t.Fail() }
}

func TestFollowing(t *testing.T) {
  dict := InsertIntoDictionary()
  following := dict.Following("ab")
  if len(following) != 2 { t.Fail() }
  if following[0] != 'b' { t.Fail() }
  if following[1] != 'r' { t.Fail() }
}

