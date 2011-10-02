// Basic trie data structure with additional prefix-checking functionality.

package trie

type Trie struct {
  terminal bool
  children map[byte]*Trie
}

func New() *Trie {
  trie := new(Trie)
  trie.children = make(map[byte] *Trie)
  return trie
}

func (self* Trie) Insert(word string) {
  if len(word) == 0 {
    self.terminal = true
    return
  }
  child, existing := self.children[word[0]]
  if !existing {
    child = New()
    self.children[word[0]] = child
  }
  child.Insert(word[1:])
}

func (self* Trie) Find(word string) bool {
  cur := self
  existing := false
  for i := 0; i < len(word); i++ {
    letter := word[i]
    // TODO(ariw): Remove hack.
    if letter < 'A' { letter += 26 }
    cur, existing = cur.children[letter]
    if !existing {
      return false
    }
  }
  return cur.terminal
}

// Return a list of characters that follow the given prefix.
func (self* Trie) Following(prefix string) (following []byte) {
  cur := self
  existing := false
  for i := 0; i < len(prefix); i++ {
    letter := prefix[i]
    // TODO(ariw): Remove hack.
    if letter < 'A' { letter += 26 }
    cur, existing = cur.children[letter]
    if !existing {
      return
    }
  }
  following = make([]byte, len(cur.children))
  i := 0
  for key, _ := range(cur.children) {
    following[i] = key
    i++
  }
  return
}

