package scrabble

import ("testing")

var strings = []string{
  "abra"
  "existing"
  "textual"
  "later"
}

func TestInsertAndRetrieve(t *testing.T) {
  dict := New()
  for i := 0; i < len(strings); i++ {
    dict.Insert(strings[i])
  }
  for i := 0; i < len(strings); i++ {
    if !dict.Find(strings[i]) {
      t.Errorf("Could not find %s in dict.", strings[i])
    }
  }
}

func TestNonExistent(t *testing.T) {
}

func TestFollowing(t *testing.T) {
}

