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

// Hash function for Moves.
func (self *Move) Hash() uint32 {
   return crc32.ChecksumIEEE(
       []byte(string([]int{self.Start.X, self.Start.Y, int(self.Direction)}) +
              self.Word))
}

// Equality function for Moves.
func (self *Move) Equals(other *Move) bool {
  return self.Start.X == other.Start.X && self.Start.Y == other.Start.Y &&
         self.Direction == other.Direction && self.Word == other.Word
}

// Used to sort vectors of Move objects by score.
func Greater(a, b interface{}) bool {
  c := a.(Move)
  d := b.(Move)
  if c.Score > d.Score {
    return true
  } else if c.Score < d.Score {
    return false
  } else if c.Word < d.Word {
    return true
  }
  return false
}

func MoveWord(move *Move) (string) {
  word := make([]byte, len(move.Word))
  for i := 0; i < len(move.Word); i++ {
    if move.Word[i] >= 'A' {
      word[i] = move.Word[i]
    } else {
      word[i] = move.Word[i] + 26 - 'A' + 'a'
    }
  }
  return string(word)
}

func PrintMove(move *Move) {
  var direction string
  if (move.Direction == ACROSS) {
    direction = "across"
  } else {
    direction = "down"
  }

  fmt.Printf("%s, worth %d points, starting at %d, %s, going %s.\n",
             MoveWord(move), move.Score, move.Start.X + 1,
             string(move.Start.Y + 'A'), direction)
}

