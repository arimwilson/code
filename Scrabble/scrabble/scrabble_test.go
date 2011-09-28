package scrabble_test

import ("testing";
        "moves"; "scrabble"; "sort_with"; "util")

func TestBlankScore(t *testing.T) {
  if scrabble.BlankScore(10, 5, '-') != 5 {
    t.Fail()
  }
  if scrabble.BlankScore(10, 3, '1') != 4 {
    t.Fail()
  }
  if scrabble.BlankScore(10, 2, '3') != 3 {
    t.Fail()
  }
}

func TestCanFollow(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  if !scrabble.CanFollow(dict, "AB", map[byte] int {'R': 1}) {
    t.Fail()
  } else if scrabble.CanFollow(dict, "AB", map[byte] int {'C': 1}) {
    t.Fail()
  }
}

func TestGetMoveList(t *testing.T) {
  dict := util.TestInsertIntoDictionary()
  board := [][]byte{
      []byte("4---2--3--2---4"),
      []byte("-3---4---4---3-"),
      []byte("--1---3-3---1--"),
      []byte("---4---1---4---"),
      []byte("2---1-3-3-1---2"),
      []byte("-4---4---4---4-"),
      []byte("--3-3-----3-3--"),
      []byte("3--1---A---1--3"),
      []byte("--3-3-----3-3--"),
      []byte("-4---4---4---4-"),
      []byte("2---1-3-3-1---2"),
      []byte("---4---1---4---"),
      []byte("--1---3-3---1--"),
      []byte("-3---4---4---3-"),
      []byte("4---2--3--2---4")}
      tiles := map[byte] int{'A': 1, 'B': 2, 'R': 1}
  letterValues := map[byte] int{'A': 1, 'B': 1, 'R': 2}
  crossChecks := make(map[int] map[byte] int)

  comparedMoves := []moves.Move {
    moves.Move{"ABRA", 5, moves.Location{7, 4}, moves.ACROSS},
    moves.Move{"ABRA", 5, moves.Location{7, 7}, moves.ACROSS},
    moves.Move{"ABBA", 4, moves.Location{7, 4}, moves.ACROSS},
    moves.Move{"ABBA", 4, moves.Location{7, 7}, moves.ACROSS}}

  moveList := scrabble.GetMoveList(
    dict, board, tiles, letterValues, crossChecks)
  sort_with.SortWith(*moveList, moves.Greater)
  util.RemoveDuplicates(moveList)
  if moveList.Len() != 4 {
    t.Fail()
  }
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    if !move.Equals(&comparedMoves[i]) {
      t.Fail()
    }
  }
}

