package cross_check_test

import ("testing";
        "scrabblish/cross_check"; "scrabblish/moves"; "scrabblish/trie";
        "scrabblish/util")

func TestGetCrossChecks(t *testing.T) {
  util.BOARD_SIZE = 5
  dict := trie.New()
  dict.Insert("ABRA")
  dict.Insert("BOO")
  dict.Insert("CHIT")
  dict.Insert("AB")

  board := [][]byte{
      []byte("-----"),
      []byte("-----"),
      []byte("ABRA-"),
      []byte("-----"),
      []byte("-----")}

  letterValues := make(map[byte] int)
  for i := byte(0); i < byte(26); i++ {
    letterValues['A' + i] = 1
  }

  crossChecks := cross_check.GetCrossChecks(
      dict, util.Transpose(board), letterValues)

  if (len(crossChecks) != 8) { t.Fatal() }
  location := moves.Location{1, 0}
  positionCrossChecks, existing := crossChecks[location.Hash()]
  if !existing || len(positionCrossChecks) != 0 { t.Fail() }
  location = moves.Location{1, 1}
  positionCrossChecks, existing = crossChecks[location.Hash()]
  score, tileExisting := positionCrossChecks['A']
  if !existing || len(positionCrossChecks) != 1 || !tileExisting ||
     score != 2 {
    t.Fail()
  }
  for i := 2; i <= 3; i++ {
    location = moves.Location{1, i}
    positionCrossChecks, existing = crossChecks[location.Hash()]
    if !existing || len(positionCrossChecks) != 0 { t.Fail() }
  }
  location = moves.Location{3, 0}
  positionCrossChecks, existing = crossChecks[location.Hash()]
  score, tileExisting = positionCrossChecks['B']
  if !existing || len(positionCrossChecks) != 1 || !tileExisting ||
     score != 2 {
    t.Fail()
  }
  for i := 1; i <= 2; i++ {
    location = moves.Location{3, i}
    positionCrossChecks, existing = crossChecks[location.Hash()]
    if !existing || len(positionCrossChecks) != 0 { t.Fail() }
  }
  location = moves.Location{3, 3}
  positionCrossChecks, existing = crossChecks[location.Hash()]
  score, tileExisting = positionCrossChecks['B']
  if !existing || len(positionCrossChecks) != 1 || !tileExisting ||
     score != 2 {
    t.Fail()
  }

  downCrossChecks := cross_check.GetCrossChecks(dict, board, letterValues)

  if (len(downCrossChecks) != 1) { t.Fatal() }
  location = moves.Location{4, 2}
  positionCrossChecks, existing = downCrossChecks[location.Hash()]
  if !existing || len(positionCrossChecks) != 0 { t.Fail() }
  util.BOARD_SIZE = 15
}

