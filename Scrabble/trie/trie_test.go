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
  if (dict.Find("ari")) {
    t.Errorf("Found ari in the dictionary.")
  }
  if (dict.Find("xylophone")) {
    t.Errorf("Found xylophone in the dictionary.")
  }
}

func TestFollowing(t *testing.T) {
  dict := InsertIntoDictionary()
  following := dict.Following("ab")
  if len(following) != 2 {
    t.Errorf("2 following characters after ab.")
  }
  if following[0] != 'b' {
    t.Errorf("b should've followed ab.")
  }
  if following[1] != 'r' {
    t.Errorf("r should've followed ab.")
  }
}

