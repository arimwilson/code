package scrabble_test

import ("os"; "sort"; "testing"; "../cross_check"; "../moves"; "../scrabble";
        "../trie"; "../util")

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

func TestGetMoveListAcross(t *testing.T) {
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
  crossChecks := cross_check.GetCrossChecks(
      dict, util.Transpose(board), letterValues)

  comparedMoves := []moves.Move {
      moves.Move{Word: "ABRA", Score: 5, Start: moves.Location{7, 4},
                 Direction: moves.ACROSS},
      moves.Move{Word: "ABRA", Score: 5, Start: moves.Location{7, 7},
                 Direction: moves.ACROSS},
      moves.Move{Word: "ABBA", Score: 5, Start: moves.Location{7, 4},
                 Direction: moves.ACROSS},
      moves.Move{Word: "ABBA", Score: 5, Start: moves.Location{7, 7},
                 Direction: moves.ACROSS}}

  moveList := scrabble.GetMoveListAcross(
    dict, board, tiles, letterValues, 40, crossChecks)
  sort.Sort(moves.Moves(moveList))
  util.RemoveDuplicates(&moveList)
  if len(moveList) != 4 {
    util.PrintMoveList(moveList, board, 2)
    t.Fatalf("length of move list: %d, should have been: 4", len(moveList))
  }
  for i := 0; i < len(moveList); i++ {
    move := moveList[i]
    if !move.Equals(&comparedMoves[i]) {
      moves.PrintMove(&move)
      moves.PrintMove(&comparedMoves[i])
      t.Fatalf("move does not equal compared move")
    }
  }
}

func prepareRealData(tilesFlag string) (
    dict *trie.Trie, tiles map[byte] int, letterValues map[byte] int) {
  wordListFile, err := os.Open("twl.txt");
  defer wordListFile.Close();
  if err != nil {
    panic("could not open twl.txt successfully.")
  }
  dict = util.ReadWordList(wordListFile)
  tiles = util.ReadTiles(tilesFlag)
  letterValues = util.ReadLetterValues(
      "1 4 4 2 1 4 3 4 1 10 5 1 3 1 1 4 10 1 1 1 2 4 4 8 4 10")
  return
}

func numTotalTopMoves(
    t *testing.T, board [][]byte, tilesFlag string, num int, score int,
    numTop int) {
  dict, tiles, letterValues := prepareRealData(tilesFlag)
  moveList := scrabble.GetMoveList(dict, board, tiles, letterValues, 40)
  if len(moveList) != num {
    util.PrintMoveList(moveList, board, 2)
    t.Errorf("length of move list: %d, should have been: %d", len(moveList),
             num)
  }
  topMove := moveList[0]
  topMoveScore := topMove.Score
  i := 1
  for ; i < len(moveList) && moveList[i].Score == topMoveScore;
      i++ {}
  if topMoveScore != score {
    moves.PrintMove(&topMove)
    t.Errorf("top move score: %d, should have been: %d", topMoveScore, score)
  } else if i != numTop {
    t.Errorf("number of top moves: %d, should have been: %d", i,
             numTop)
  }
}

func getComplicatedBoard() (board [][]byte) {
  board = [][]byte{
      []byte("4---2--3--2---4"),
      []byte("-3---4---4---3-"),
      []byte("--1---3-3---1--"),
      []byte("---4---1---4---"),
      []byte("2---1-3-LIKE--2"),
      []byte("-4--LOTTO4---4-"),
      []byte("--3-3---V-3-3--"),
      []byte("3--1-FACED-1--3"),
      []byte("--3-3-R---3-3--"),
      []byte("-4---4ERA4---4-"),
      []byte("2---1-3-R-1---2"),
      []byte("---4---1K--4---"),
      []byte("--1---3-3---1--"),
      []byte("-3---4---4---3-"),
      []byte("4---2--3--2---4")}
  return
}

func TestNumTotalTopMoves(t *testing.T) {
  board := [][]byte{
      []byte("4---2--3--2---4"),
      []byte("-3---4---4---3-"),
      []byte("--1---3-3---1--"),
      []byte("---4---1---4---"),
      []byte("2---1-3-3-1---2"),
      []byte("-4---4---4---4-"),
      []byte("--3-3-----3-3--"),
      []byte("3--1---*---1--3"),
      []byte("--3-3-----3-3--"),
      []byte("-4---4---4---4-"),
      []byte("2---1-3-3-1---2"),
      []byte("---4---1---4---"),
      []byte("--1---3-3---1--"),
      []byte("-3---4---4---3-"),
      []byte("4---2--3--2---4")}
  numTotalTopMoves(t, board, "ABCDEFG", 346, 24, 8)
  numTotalTopMoves(t, board, "ABCDEF ", 4816, 28, 8)
  board[7] = []byte("3--1-FACED-1--3")
  numTotalTopMoves(t, board, "ABCDEFG", 337, 34, 1)
  board[8][7] = byte('A')
  board[9][7] = byte('R')
  numTotalTopMoves(t, board, "ABCDEFR", 777, 45, 3)
  numTotalTopMoves(t, getComplicatedBoard(), "ABCDEF ", 4903, 90, 1)
}

func BenchmarkAll(b *testing.B) {
  for i := 0; i < b.N; i++ {
    dict, tiles, letterValues := prepareRealData("ABCDEFG")
    scrabble.GetMoveList(dict, getComplicatedBoard(), tiles, letterValues, 40)
  }
}

func BenchmarkGetMoveList(b *testing.B) {
  b.StopTimer()
  dict, tiles, letterValues := prepareRealData("ABCDEFG")
  board := getComplicatedBoard()
  b.StartTimer()
  for i := 0; i < b.N; i++ {
    scrabble.GetMoveList(dict, board, tiles, letterValues, 40)
  }
}

