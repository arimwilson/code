// Basic trie data structure with additional following functionality.

package trie

import ("fmt")

type Trie struct {
  Terminal bool
  Children map[byte]*Trie
}

// Make a new trie.
func New() *Trie {
  trie := new(Trie)
  trie.Terminal = false
  trie.Children = nil
  return trie
}

// Insert a word into the trie.
func (self *Trie) Insert(word string) {
  cur := self
  for i := 0; i < len(word); i++ {
    if cur.Children == nil {
      cur.Children = make(map[byte] *Trie)
    }
    child, existing := cur.Children[word[i]]
    if !existing {
      child = New()
      cur.Children[word[i]] = child
    }
    cur = child
  }
  cur.Terminal = true
}

// Return the children data structure (if it exists) from following the trie
// through prefix.
func (self *Trie) following(prefix string) (existing bool, cur *Trie) {
  cur = self
  for i := 0; i < len(prefix); i++ {
    letter := prefix[i]
    // TODO(ariw): Remove hack.
    if letter < 'A' { letter += 26 }
    existing = false
    if cur.Children == nil { return }
    cur, existing = cur.Children[letter]
    if !existing { return }
  }
  return
}

// Whether or not a word exists in the trie.
func (self *Trie) Find(word string) bool {
  existing, cur := self.following(word)
  return existing && cur.Terminal
}

// Return a list of characters that follow the given prefix.
func (self *Trie) Following(prefix string) (following []byte) {
  existing, cur := self.following(prefix)
  if !existing || cur.Children == nil { return }
  following = make([]byte, len(cur.Children))
  i := 0
  for key, _ := range(cur.Children) {
    following[i] = key
    i++
  }
  return
}

func (self *Trie) print(n int) {
  if self.Children == nil { return }
  for key, value := range(self.Children) {
    for i := 0; i < n; i++ {
      fmt.Printf(" ")
    }
    fmt.Printf(string(key) + "\n")
    value.print(n + 1)
  }
}

// Print a trie.
func (self *Trie) Print() {
  self.print(0)
}

