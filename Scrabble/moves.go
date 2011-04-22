// Move data structures to keep track of valid moves.

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

// Used to sort vectors of Move objects by score.
func Less(a, b interface{}) bool {
  return a.(Move).Score < b.(Move).Score
}

