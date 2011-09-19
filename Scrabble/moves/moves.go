// Move data structures to keep track of valid moves.

package moves

import ("fmt";
        "hash/crc32")

type Direction int; const (
  ACROSS = iota
  DOWN
)

type Location struct {
  X int
  Y int
}

// Hash function for Locations.
func (self* Location) Hash() int {
  return int(crc32.ChecksumIEEE([]byte(string([]int{self.X, self.Y}))))
}

type Move struct {
  Word string
  Score int
  Start Location
  Direction Direction
}

// Used to sort vectors of Move objects by score.
func Greater(a, b interface{}) bool {
  return a.(Move).Score > b.(Move).Score
}

func PrintMove(move *Move) {
  var direction string
  if (move.Direction == ACROSS) {
    direction = "across"
  } else {
    direction = "down"
  }
  fmt.Printf("%s, worth %d points, starting at %d, %d, going %s.\n",
             move.Word, move.Score, move.Start.X, move.Start.Y, direction)
}

