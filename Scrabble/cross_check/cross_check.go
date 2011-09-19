package cross_check

import ("fmt"; "strings";
        "moves"; "trie"; "util")

// Entry in cross check set means some tiles are allowable vertically, with
// given point values. No entry means all tiles are allowable for no points.
func GetCrossChecks(
    dict *trie.Trie, transposedBoard [][]byte, tiles map[byte] int,
    letterValues map[byte] int) (crossChecks map[int] map[byte] int) {
  crossChecks = make(map[int] map[byte] int)
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      if util.Available(transposedBoard, &moves.Location{i, j}) {
        // Go left and see if there's a word.
        l := j - 1
        for ; l >= 0 && !util.Available(transposedBoard, &moves.Location{i, l});
            l-- {}
        // Go right and see if there's a word.
        r := j + 1
        for ; r < util.BOARD_SIZE &&
              !util.Available(transposedBoard, &moves.Location{i, r}); r++ {}
        if l == j - 1 && r == j + 1 { continue }
        possibleMove := moves.Move{Score: 0, Start: moves.Location{i, j},
                                   Direction: moves.ACROSS }
        location := moves.Location{j, i}
        if _, existing := crossChecks[location.Hash()]; existing {
          // Bad hash, panic!
          panic(fmt.Sprintf("Existing cross-check for position %d, %d!", j, i))
        }
        crossChecks[location.Hash()] = make(map[byte] int)
        positionCrossChecks := crossChecks[location.Hash()]
        sides := []string{"", "", ""}
        if l < j -1 {
          possibleMove.Start.X = l + 1
          sides[0] = string(transposedBoard[i][l + 1:j])
        }
        if r > j + 1 {
          sides[2] = string(transposedBoard[i][j + 1:r])
        }
        for tile, _ := range tiles {
          sides[1] = string(tile)
          possibleMove.Word = strings.Join(sides, "")
          if (dict.Find(possibleMove.Word)) {
            util.Score(transposedBoard, letterValues, &possibleMove)
            positionCrossChecks[tile] = possibleMove.Score
          }
        }
      }
    }
  }
  return
}

func PrintCrossChecks(crossChecks map[int] map[byte] int) {
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      location := moves.Location{i, j}
      positionCrossChecks, existing := crossChecks[location.Hash()]
      if existing {
        fmt.Printf("%d, %d: ", i, j)
        for letter, score := range(positionCrossChecks) {
          fmt.Printf("%c %d", letter, score)
        }
        fmt.Printf("\n")
      }
    }
  }
}

