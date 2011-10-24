// Basic trie data structure with additional following functionality.

package trie

import ("fmt")

type Child struct {
  Letter byte
  Trie *Trie
}

type Trie struct {
  Terminal bool
  Children []Child
}

// Make a new trie.
func New() *Trie {
  trie := new(Trie)
  return trie
}

func findChild(children []Child, letter byte) (trie *Trie, existing bool) {
  existing = false
  for i := 0; i < len(children); i++ {
    if children[i].Letter == letter {
      trie = children[i].Trie
      existing = true
      return
    }
  }
  return
}

// Insert a word into the trie.
func (self *Trie) Insert(word string) {
  if len(word) == 0 {
    self.Terminal = true
    return
  }
  child, existing := findChild(self.Children, word[0])
  if !existing {
    child = New()
    self.Children = append(self.Children, Child{word[0], child})
  }
  child.Insert(word[1:])
}

// Return the children data structure (if it exists) from following the trie
// through prefix.
func (self *Trie) following(prefix string) (cur *Trie, existing bool) {
  cur = self
  for i := 0; i < len(prefix); i++ {
    letter := prefix[i]
    // TODO(ariw): Remove hack.
    if letter < 'A' { letter += 26 }
    cur, existing = findChild(cur.Children, letter)
    if !existing { return }
  }
  return
}

// Whether or not a word exists in the trie.
func (self *Trie) Find(word string) bool {
  cur, existing := self.following(word)
  return existing && cur.Terminal
}

// Return a list of characters that follow the given prefix.
func (self *Trie) Following(prefix string) (following []byte) {
  cur, existing := self.following(prefix)
  if !existing { return }
  following = make([]byte, len(cur.Children))
  for i := 0; i < len(cur.Children); i++ {
    following[i] = cur.Children[i].Letter
  }
  return
}

func (self *Trie) print(n int) {
  if self.Children == nil { return }
  for i := 0; i < len(self.Children); i++ {
    for i := 0; i < n; i++ {
      fmt.Printf(" ")
    }
    fmt.Printf(string(self.Children[i].Letter) + "\n")
    self.Children[i].Trie.print(n + 1)
  }
}

// Print a trie.
func (self *Trie) Print() {
  self.print(0)
}

