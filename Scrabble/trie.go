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

func (trie* Trie) Insert(word string) {
  if len(word) == 0 {
    trie.terminal = true
    return
  }
  child, ok := trie.children[word[0]]
  if !ok {
    child = New()
    trie.children[word[0]] = child
  }
  child.Insert(word[1:])
}

func (trie* Trie) Find(word string) (bool) {
  if len(word) == 0 {
    if trie.terminal {
      return true
    } else {
      return false
    }
  }
  child, ok := trie.children[word[0]]
  if ok {
    return child.Find(word[1:])
  }
  return false
}

