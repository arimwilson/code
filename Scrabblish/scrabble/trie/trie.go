// Basic trie data structure with additional following functionality.

package trie

type Trie struct {
  terminal bool
  children map[byte]*Trie
}

// Make a new trie.
func New() *Trie {
  trie := new(Trie)
  trie.children = make(map[byte] *Trie)
  return trie
}

// Insert a word into the trie.
func (self *Trie) Insert(word string) {
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

// Return the children data structure (if it exists) from following the trie
// through prefix.
func (self *Trie) following(prefix string) (cur *Trie, existing bool) {
  cur = self
  for i := 0; i < len(prefix); i++ {
    letter := prefix[i]
    // TODO(ariw): Remove hack.
    if letter < 'A' { letter += 26 }
    cur, existing = cur.children[letter]
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
  i := 0
  for key, _ := range(cur.children) {
    following[i] = key
    i++
  }
  return
}

