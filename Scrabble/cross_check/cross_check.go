package cross_check

import ("util")

type CrossCheck struct {
  letter byte
  score int
}

// Entry in cross check set means some tiles are allowable vertically, with
// given point values. No entry means all tiles are allowable for no points.
func GetCrossChecks(dict *trie.Trie, board [][]byte, tiles map[byte] int,
                    letterValues map[byte] int)
    (crossChecks  map[moves.Location] {
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      if !util.Existing(board, &moves.Location({i, j})) {
        // Go up and see if there's a word.
      }
    }
  }
}

