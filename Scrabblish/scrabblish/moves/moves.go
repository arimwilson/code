// Move data structures to keep track of valid moves.

package moves

import ("bytes";
        "encoding/binary";
        "fmt";
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
   buf := new(bytes.Buffer)
   _ = binary.Write(buf, binary.LittleEndian, []int{self.X, self.Y})
  return int(crc32.ChecksumIEEE(buf.Bytes()))
}

type Move struct {
  Word string
  Score int
  Start Location
  Direction Direction
}

func (self* Move) Copy() Move {
  newMove := new(Move)
  *newMove = *self
  return *newMove
}

// Hash function for Moves.
func (self *Move) Hash() uint32 {
   buf := new(bytes.Buffer)
   _ = binary.Write(buf, binary.LittleEndian,
                    []int{self.Start.X, self.Start.Y, int(self.Direction)})
   _ = binary.Write(buf, binary.LittleEndian,
                    self.Word)
   return crc32.ChecksumIEEE(buf.Bytes())
}

// Equality function for Moves.
func (self *Move) Equals(other *Move) bool {
  return self.Start.X == other.Start.X && self.Start.Y == other.Start.Y &&
         self.Direction == other.Direction && self.Word == other.Word
}

// Used to sort vectors of Move objects by score.
type Moves []Move

func (moves Moves) Len() int {
  return len(moves)
}

func (moves Moves) Swap(i, j int) {
  moves[i], moves[j] = moves[j], moves[i]
}

// Actually Greater, named Less for the purposes of the Sort interface.
func (moves Moves) Less(i, j int) bool {
  a := moves[i]
  b := moves[j]
  if a.Score > b.Score {
    return true
  } else if a.Score < b.Score {
    return false
  } else if a.Word < b.Word {
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

func PrintMove(move *Move) string {
  var direction string
  if (move.Direction == ACROSS) {
    direction = "across"
  } else {
    direction = "down"
  }

  return fmt.Sprintf(
    "%s, worth %d points, starting at %d, %s, going %s.\n", MoveWord(move),
    move.Score, move.Start.X + 1, string(move.Start.Y + 'A'), direction)
}

