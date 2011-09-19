package cross_check_test

import ("testing";
        "cross_check"; "moves"; "trie"; "util")

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

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

  tiles := make(map[byte] int)
  tiles['B'] = 1
  tiles['O'] = 2

  letterValues := make(map[byte] int)
  for i := 0; i < len(ALPHABET); i++ {
    letterValues[ALPHABET[i]] = 1
  }

  crossChecks := cross_check.GetCrossChecks(
      dict, util.Transpose(board), tiles, letterValues)

  if (len(crossChecks) != 8) { t.Fatal() }
  for i := 0; i <= 3; i++ {
    location := moves.Location{1, i}
    positionCrossChecks, existing := crossChecks[location.Hash()]
    if !existing || positionCrossChecks.Len() != 0 { t.Fail() }
  }
  location := moves.Location{3, 0}
  positionCrossChecks, existing := crossChecks[location.Hash()]
  if !existing || positionCrossChecks.Len() != 1 ||
     positionCrossChecks.At(0).(*cross_check.PositionCrossCheck).Letter != 'B' {
    t.Fail()
  }
  for i := 1; i <= 2; i++ {
    location = moves.Location{3, i}
    positionCrossChecks, existing = crossChecks[location.Hash()]
    if !existing || positionCrossChecks.Len() != 0 { t.Fail() }
  }
  location = moves.Location{3, 3}
  positionCrossChecks, existing = crossChecks[location.Hash()]
  if !existing || positionCrossChecks.Len() != 1 ||
     positionCrossChecks.At(0).(*cross_check.PositionCrossCheck).Letter != 'B' {
    t.Fail()
  }

  downCrossChecks := cross_check.GetCrossChecks(
      dict, board, tiles, letterValues)

  if (len(downCrossChecks) != 1) { t.Fatal() }
  location = moves.Location{4, 2}
  positionCrossChecks, existing = downCrossChecks[location.Hash()]
  if !existing || positionCrossChecks.Len() != 0 { t.Fail() }
}

