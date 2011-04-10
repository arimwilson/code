// Basic trie data structure.

package trie

type Trie struct {
  Terminal bool
  Children map[byte]*Trie
}

func New() *Trie {
  trie := new(Trie)
  trie.Terminal = false
	trie.Children = make(map[byte]*Trie)
	return trie
}

func (trie *Trie) Insert(Word string) {
  if len(Word) == 0 {
    trie.Terminal = true
    return
  }
  child, ok := trie.Children[Word[0]]
  if !ok {
    child = New()
    trie.Children[Word[0]] = child
  }
  child.Insert(Word[1:])
}

func (trie *Trie) Find(Word string) (bool) {
  if len(Word) == 0 {
    if trie.Terminal {
      return true
    } else {
      return false
    }
  }
  child, ok := trie.Children[Word[0]]
  if ok {
    return child.Find(Word[1:])
  }
  return false
}

