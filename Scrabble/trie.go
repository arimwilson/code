// Basic trie data structure with additional prefix-checking functionality.

package trie

type Trie struct {
  terminal bool
  children map[byte]*Trie
}

func New() *Trie {
  trie := new(Trie)
  trie.children = make(map[byte]*Trie)
  return trie
}

func (self* Trie) Insert(word string) {
  if len(word) == 0 {
    self.terminal = true
    return
  }
  child, ok := self.children[word[0]]
  if !ok {
    child = New()
    self.children[word[0]] = child
  }
  child.Insert(word[1:])
}

func (self* Trie) Find(word string) bool {
  if len(word) == 0 {
    if self.terminal {
      return true
    } else {
      return false
    }
  }
  child, ok := self.children[word[0]]
  if ok {
    return child.Find(word[1:])
  }
  return false
}

// Return a list of characters that follow the given prefix.
func (self* Trie) Following(prefix string) (following []byte) {
  cur := self
  for i := 0; i < len(prefix); i++ {
    cur = cur.children[prefix[i]]
    if cur == nil {
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

