// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package main

import ("container/vector"; "flag"; "fmt"; "os";
        "cross_check"; "moves"; "sort_with"; "trie"; "util")

var wordListFlag = flag.String(
    "w", "",
    "File with space-separated list of legal words, in upper-case.")
var boardFlag = flag.String(
    "b", "",
    "File with board structure. Format: * indicates starting point, 1 and 2 " +
    "indicate double and triple word score tiles, 3 and 4 indicate double " +
    "and triple letter score tiles, - indicates blank tiles, and upper-case " +
    "letters indicate existing words.")
var tilesFlag = flag.String(
    "t", "", "List of all 7 player tiles, in upper-case.")
var letterValuesFlag = flag.String(
    "l", "1 3 3 2 1 4 2 4 1 8 5 1 3 1 1 3 10 1 1 1 1 4 4 8 4 10",
    "Space-separated list of letter point values, from A-Z.")

// Retrieve whether or not we have tiles that can possibly follow prefix in dict.
func canFollow(dict *trie.Trie, prefix string, tiles map[byte] int) bool {
  following := dict.Following(prefix)
  for i := 0; i < len(following); i++ {
    count, existing := tiles[following[i]]
    if existing && count > 0 { return true }
  }
  return false
}

func moveRight(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, crossChecks map[int] map[byte] int,
    possibleMove moves.Move) {
  prefixEnd := moves.Location{possibleMove.Start.X, possibleMove.Start.Y + 1}
  for ; prefixEnd.Y < util.BOARD_SIZE && !util.Available(board, &prefixEnd);
      prefixEnd.Y++ {
    possibleMove.Word += string(board[prefixEnd.X][prefixEnd.Y])
  }
  if canFollow(dict, possibleMove.Word, tiles) {
    extend(dict, board, tiles, letterValues, crossChecks, possibleMove,
           moves.RIGHT)
  }
}

func extend(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, crossChecks map[int] map[byte] int,
    possibleMove moves.Move,
    direction moves.Direction) (moveList *vector.Vector) {
  moveList = new(vector.Vector)
  if (direction == moves.LEFT) {
    if (!util.Available(board, &possibleMove.Start)) {
      return
    }
    positionCrossChecks, existing := crossChecks[possibleMove.Start.Hash()]
    for tile, count := range(tiles) {
      score, tileExisting := positionCrossChecks[tile]
      if count > 0 && (!existing || tileExisting) {
        possibleMove.Word = string(tile) + possibleMove.Word
        if tileExisting { possibleMove.Score += score }
        if dict.Find(possibleMove.Word) {
          util.Score(board, letterValues, &possibleMove)
          moveList.Push(possibleMove)
        }
        tiles[tile]--
        moveRight(dict, board, tiles, letterValues, crossChecks, possibleMove)
        possibleMove.Start.Y--
        extend(dict, board, tiles, letterValues, crossChecks, possibleMove,
               moves.LEFT)
        possibleMove.Start.Y++
        tiles[tile]++
      }
    }
  } else if (direction == moves.RIGHT) {
    // TODO(ariw): Reduce duplication with extending left?
    endLocation := moves.Location{possibleMove.Start.X,
                                  possibleMove.Start.Y + len(possibleMove.Word)}
    if (!util.Available(board, &endLocation)) {
      return
    }
    positionCrossChecks, existing := crossChecks[endLocation.Hash()]
    for tile, count := range(tiles) {
      score, tileExisting := positionCrossChecks[tile]
      if count > 0 && (!existing || tileExisting) {
        possibleMove.Word += string(tile)
        if tileExisting { possibleMove.Score += score }
        if dict.Find(possibleMove.Word) {
          util.Score(board, letterValues, &possibleMove)
          moveList.Push(possibleMove)
        }
        tiles[tile]--
        moveRight(dict, board, tiles, letterValues, crossChecks, possibleMove)
        tiles[tile]++
      }
    }
  }
  return
}

func getMoveList(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int,
    crossChecks map[int] map[byte] int) (moveList vector.Vector) {
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      possibleMove := moves.Move{
        Word: "", Score: 0, Start: moves.Location{i, j},
        Direction: moves.RIGHT }
      if board[i][j] == '*' {
        moveList.AppendVector(extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            moves.LEFT))
      } else if board[i][j] >= 'A' && board[i][j] < 'Z' {
        possibleMove.Start.X--
        moveList.AppendVector(extend(
            dict, board, tiles, letterValues, crossChecks,
            possibleMove, moves.LEFT))
        possibleMove.Start.X += 2
        moveList.AppendVector(extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            moves.LEFT))
        possibleMove.Start.X--; possibleMove.Start.Y--
        moveList.AppendVector(extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            moves.LEFT))
        possibleMove.Start.Y += 2
        moveList.AppendVector(extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            moves.LEFT))
      }
    }
  }
  return
}

func setDirection(direction moves.Direction, moveList *vector.Vector) {
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    move.Direction = direction
    moveList.Set(i, move)
  }
}

func main() {
  flag.Parse()
  wordListFile, err := os.Open(*wordListFlag);
  defer wordListFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -w, found %s\n", *wordListFlag)
    os.Exit(1)
  }
  boardFile, err := os.Open(*boardFlag);
  defer boardFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -b, found %s\n", *boardFlag)
    os.Exit(1)
  }
  if len(*tilesFlag) != 7 {
    fmt.Printf("need 7 tiles in -t, found %d\n", len(*tilesFlag))
    os.Exit(1)
  }
  dict := util.ReadWordList(wordListFile)
  board := util.ReadBoard(boardFile)
  tiles := util.ReadTiles(*tilesFlag)
  letterValues := util.ReadLetterValues(*letterValuesFlag)
  transposedBoard := util.Transpose(board)

  // Get moves going both right and down.
  crossChecks := cross_check.GetCrossChecks(dict, transposedBoard, tiles,
                                            letterValues)
  moveList := getMoveList(dict, board, tiles, letterValues, crossChecks)
  setDirection(moves.RIGHT, &moveList)
  downCrossChecks := cross_check.GetCrossChecks(dict, board, tiles,
                                                letterValues)
  downMoveList := getMoveList(dict, transposedBoard, tiles, letterValues,
                              downCrossChecks)
  setDirection(moves.DOWN, &downMoveList)
  moveList.AppendVector(&downMoveList)
  sort_with.SortWith(moveList, moves.Less)
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    fmt.Printf("%d. %s, worth %d points, starting at %d, %d, going %d.",
               i, move.Word, move.Score, move.Start.X, move.Start.Y,
               move.Direction)
  }
}

