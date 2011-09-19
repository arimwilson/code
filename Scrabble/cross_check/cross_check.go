package cross_check

import ("container/vector"; "fmt"; "strings";
        "moves"; "trie"; "util")

type PositionCrossCheck struct {
  Letter byte
  Score int
}

// Entry in cross check set means some tiles are allowable vertically, with
// given point values. No entry means all tiles are allowable for no points.
func GetCrossChecks(
    dict *trie.Trie, transposedBoard [][]byte, tiles map[byte] int,
    letterValues map[byte] int) (crossChecks map[int] *vector.Vector) {
  crossChecks = make(map[int] *vector.Vector)
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      if !util.Existing(transposedBoard, &moves.Location{i, j}) {
        // Go left and see if there's a word.
        l := j - 1
        for ; l >= 0 && util.Existing(transposedBoard, &moves.Location{i, l});
            l-- {}
        // Go right and see if there's a word.
        r := j + 1
        for ; r < util.BOARD_SIZE &&
              util.Existing(transposedBoard, &moves.Location{i, r}); r++ {}
        if l == j - 1 && r == j + 1 { continue }
        transposedLocation := moves.Location{j, i}
        if _, existing := crossChecks[transposedLocation.Hash()]; existing {
          // Bad hash, panic!
          panic(fmt.Sprintf("Existing cross-check for position %d, %d!", j, i))
        }
        crossChecks[transposedLocation.Hash()] = new(vector.Vector)
        positionCrossChecks := crossChecks[transposedLocation.Hash()]
        sides := []string{"", "", ""}
        if l < j -1 {
          sides[0] = string(transposedBoard[i][l + 1:j])
        }
        if r > j + 1 {
          sides[2] = string(transposedBoard[i][j + 1:r])
        }
        for k, _ := range tiles {
          sides[1] = string(k)
          if (dict.Find(strings.Join(sides, ""))) {
            positionCrossCheck := new(PositionCrossCheck)
            positionCrossCheck.Letter = k
            // TODO(ariw): Fix.
            positionCrossCheck.Score = 1
            positionCrossChecks.Push(positionCrossCheck)
          }
        }
      }
    }
  }
  return
}

func PrintCrossChecks(crossChecks map[int] *vector.Vector) {
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      location := moves.Location{i, j}
      positionCrossChecks, existing := crossChecks[location.Hash()]
      if existing {
        fmt.Printf("%d, %d: ", i, j)
        for k := 0; k < positionCrossChecks.Len(); k++ {
          positionCrossCheck := positionCrossChecks.At(k).(*PositionCrossCheck)
          fmt.Printf("%c ", positionCrossCheck.Letter)
        }
        fmt.Printf("\n")
      }
    }
  }
}

