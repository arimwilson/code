// Basic trie data structure with additional following functionality.

package trie

type Child struct {
  letter byte
  trie *Trie
}

type Trie struct {
  terminal bool
  children []Child
}

// Make a new trie.
func New() *Trie {
  trie := new(Trie)
  return trie
}

func findChild(children []Child, letter byte) (trie *Trie, existing bool) {
  existing = false
  for i := 0; i < len(children); i++ {
    if children[i].letter == letter {
      trie = children[i].trie
      existing = true
      return
    }
  }
  return
}

// Insert a word into the trie.
func (self *Trie) Insert(word string) {
  if len(word) == 0 {
    self.terminal = true
    return
  }
  child, existing := findChild(self.children, word[0])
  if !existing {
    child = New()
    self.children = append(self.children, Child{word[0], child})
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
    cur, existing = findChild(cur.children, letter)
    if !existing { return }
  }
  return
}

// Whether or not a word exists in the trie.
func (self *Trie) Find(word string) bool {
  cur, existing := self.following(word)
  return existing && cur.terminal
}

// Return a list of characters that follow the given prefix.
func (self *Trie) Following(prefix string) (following []byte) {
  cur, existing := self.following(prefix)
  if !existing { return }
  following = make([]byte, len(cur.children))
  for i := 0; i < len(cur.children); i++ {
    following[i] = cur.children[i].letter
  }
  return
}

