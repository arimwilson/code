// Move data structures to keep track of valid moves. Moves data structure is
// effectively a set.

package moves

type Direction int; const (
  RIGHT = iota
  DOWN
)

type Location struct {
  X int
  Y int
}

type Move struct {
  Word string
  Score int
  Start Location
  Direction Direction
}

type Moves []Move

func (self Moves) Len() int {
  return len(self)
}

func (self Moves) Less(i, j int) bool {
  return self[i].Score < self[j].Score
}

func (self Moves) Swap(i, j int) {
  self[i], self[j] = self[j], self[i]
}

