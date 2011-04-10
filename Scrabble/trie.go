// Basic trie data structure.

package trie

type Trie struct {
  terminal bool
  children map[byte]*Trie
}

func New() *Trie {
  trie := new(Trie)
  trie.terminal = false
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

