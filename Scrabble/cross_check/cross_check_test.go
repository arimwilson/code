package cross_check_test

import ("container/vector"; "fmt"; "testing";
        "cross_check"; "moves"; "trie"; "util")

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

func printCrossChecks(message string, crossChecks map[int] *vector.Vector) {
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      location := moves.Location{i, j}
      positionCrossChecks, existing := crossChecks[location.Hash()]
      if existing {
        fmt.Printf("%s at %d, %d: ", message, i, j)
        for k := 0; k < positionCrossChecks.Len(); k++ {
          positionCrossCheck :=
              positionCrossChecks.At(k).(*cross_check.PositionCrossCheck)
          fmt.Printf("%c ", positionCrossCheck.Letter)
          // TODO(ariw): Assert about this.
        }
        fmt.Printf("\n")
      }
    }
  }
}

func TestGetCrossChecks(t *testing.T) {
  util.BOARD_SIZE = 5
  dict := trie.New()
  dict.Insert("abra")
  dict.Insert("boo")
  dict.Insert("chit")
  dict.Insert("ab")

  board := [][]byte{
      []byte("-----"),
      []byte("-----"),
      []byte("abra-"),
      []byte("-----"),
      []byte("-----")}

  tiles := make(map[byte] int)
  tiles['b'] = 1
  tiles['o'] = 2

  letterValues := make(map[byte] int)
  for i := 0; i < len(ALPHABET); i++ {
    letterValues[ALPHABET[i]] = 1
  }

  crossChecks := cross_check.GetCrossChecks(
      dict, util.Transpose(board), tiles, letterValues)
  downCrossChecks := cross_check.GetCrossChecks(
      dict, board, tiles, letterValues)
  printCrossChecks("CrossCheck", crossChecks)
  printCrossChecks("DownCrossCheck", downCrossChecks)
}

