package scrabble

import ("container/vector";
        "scrabblish/cross_check"; "scrabblish/moves"; "scrabblish/trie";
        "scrabblish/sort_with"; "scrabblish/util")

// Your score without the points from the blank letter given from the value
// retrieved from letterValue.
func BlankScore(score int, letterValue int, tile byte) int {
  letterMultiplier, wordMultiplier := util.TileMultipliers(&tile)
  return score / wordMultiplier - letterMultiplier * letterValue
}

// Retrieve whether or not we have tiles that can possibly follow prefix in
// dict.
func CanFollow(dict *trie.Trie, prefix string, tiles map[byte] int) bool {
  following := dict.Following(prefix)
  count, existing := tiles[' ']
  if len(following) > 0 && existing && count > 0 { return true }
  for i := 0; i < len(following); i++ {
    count, existing = tiles[following[i]]
    if existing && count > 0 { return true }
  }
  return false
}

// Return byte array consisting of existing tiles on the board to the left of
// location.
func GetExistingLeftTiles(board [][]byte, location *moves.Location) string {
  end := location.Y
  if end < 0 { return "" }
  location.Y--
  for ; util.Existing(board, location); location.Y-- {}
  location.Y++
  return string(board[location.X][location.Y:end])
}

// Return byte array consisting of existing tiles on the board to the right of
// location.
func GetExistingRightTiles(board [][]byte, location *moves.Location) string {
  end := location.Y + 1
  if end >= util.BOARD_SIZE { return "" }
  for ; util.ExistingLocation(board, location.X, end); end++ {}
  return string(board[location.X][location.Y + 1:end])
}

// Add one letter to a possible move, checking if we've got a word, going either
// left or right.
func Extend(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, bonus int, crossChecks map[int] map[byte] int,
    possibleMove moves.Move, left bool, moveList *vector.Vector) {
  var positionCrossChecks map[byte] int
  var existing bool
  var placedLocation moves.Location
  var existingTiles string
  if left {
    placedLocation = possibleMove.Start
    if !util.Available(board, &placedLocation) { return }
    positionCrossChecks, existing = crossChecks[placedLocation.Hash()]
    existingTiles = GetExistingLeftTiles(board, &placedLocation)
  } else {
    placedLocation = moves.Location{
        possibleMove.Start.X, possibleMove.Start.Y + len(possibleMove.Word)}
    if !util.Available(board, &placedLocation) { return }
    positionCrossChecks, existing = crossChecks[placedLocation.Hash()]
    existingTiles = GetExistingRightTiles(board, &placedLocation)
  }
  placedMove := possibleMove
  for tile, count := range(tiles) {
    if count == 0 { continue }
    verticallyScoredLetters := make(map[byte] int)
    if tile != ' ' {
      if existing {
        score, tileExisting := positionCrossChecks[tile]
        if tileExisting { verticallyScoredLetters[tile] = score }
      } else {
        verticallyScoredLetters[tile] = 0
      }
    } else if existing {
      for i, score := range(positionCrossChecks) {
        verticallyScoredLetters[i - 26] = BlankScore(
            score, letterValues[i], board[placedLocation.X][placedLocation.Y])
      }
    } else {
      for i := 'A'; i <= 'Z'; i++ {
        verticallyScoredLetters[byte(i - 26)] = 0
      }
    }
    for letter, score := range(verticallyScoredLetters) {
      placedMove.Score = possibleMove.Score
      if left {
        placedMove.Word = existingTiles + string(letter) + possibleMove.Word
        placedMove.Start = placedLocation
      } else {
        placedMove.Word = possibleMove.Word + string(letter) + existingTiles
      }
      placedMove.Score += score
      if dict.Find(placedMove.Word) {
        score = placedMove.Score
        util.Score(board, letterValues, bonus, &placedMove)
        moveList.Push(placedMove)
        placedMove.Score = score
      }
      tiles[tile]--
      if CanFollow(dict, placedMove.Word, tiles) {
        Extend(dict, board, tiles, letterValues, bonus, crossChecks, placedMove,
               false, moveList)
      }
      if left {
        placedMove.Start.Y--
        Extend(dict, board, tiles, letterValues, bonus, crossChecks, placedMove,
               true, moveList)
      }
      tiles[tile]++
    }
  }
  return
}

// Look for new across moves connected to any existing tile. Duplicates are
// possible.
func GetMoveListAcross(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, bonus int,
    crossChecks map[int] map[byte] int) (moveList *vector.Vector) {
  moveList = new(vector.Vector)
  possibleMove := moves.Move{ Word: "", Score: 0, Direction: moves.ACROSS }
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      if board[i][j] == '*' {
        possibleMove.Start = moves.Location{ i, j }
        Extend(dict, board, tiles, letterValues, bonus, crossChecks,
               possibleMove, true, moveList)
      } else if board[i][j] >= 'A' && board[i][j] <= 'Z' {
        possibleMove.Start.X = i
        possibleMove.Start.Y = j - 1
        possibleMove.Word = GetExistingRightTiles(board, &possibleMove.Start)
        Extend(dict, board, tiles, letterValues, bonus, crossChecks, possibleMove,
               true, moveList)
        possibleMove.Start.Y = j + 1
        possibleMove.Word = GetExistingLeftTiles(board, &possibleMove.Start)
        Extend(dict, board, tiles, letterValues, bonus, crossChecks,
               possibleMove, false, moveList)
        possibleMove.Start.Y = j
        possibleMove.Word = ""
        leftUp := moves.Location{i - 1, j - 1}
        rightUp := moves.Location{i - 1, j + 1}
        if !util.Existing(board, &leftUp) && !util.Existing(board, &rightUp) {
          possibleMove.Start.X = i - 1
         Extend(dict, board, tiles, letterValues, bonus, crossChecks,
                possibleMove, true, moveList)
        }
        leftDown := moves.Location{i + 1, j - 1}
        rightDown := moves.Location{i + 1, j + 1}
        if !util.Existing(board, &leftDown) &&
           !util.Existing(board, &rightDown) {
          possibleMove.Start.X = i + 1
          Extend(dict, board, tiles, letterValues, bonus, crossChecks,
                 possibleMove, true, moveList)
        }
      }
    }
  }
  return
}

// Set Direction for all moves in moveList to direction.
func SetDirection(direction moves.Direction, moveList *vector.Vector) {
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    if move.Direction != direction {
      move.Start.X, move.Start.Y = move.Start.Y, move.Start.X
    }
    move.Direction = direction
    moveList.Set(i, move)
  }
}

// Get all possible moves on board, ordered by score, given params.
func GetMoveList(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, bonus int) (moveList *vector.Vector) {
  transposedBoard := util.Transpose(board)
  crossChecks := cross_check.GetCrossChecks(dict, transposedBoard, letterValues)
  moveList = GetMoveListAcross(dict, board, tiles, letterValues, crossChecks,
                               bonus)
  SetDirection(moves.ACROSS, moveList)
  downCrossChecks := cross_check.GetCrossChecks(dict, board, letterValues)
  downMoveList := GetMoveListAcross(
      dict, transposedBoard, tiles, letterValues, downCrossChecks, bonus)
  SetDirection(moves.DOWN, downMoveList)
  moveList.AppendVector(downMoveList)
  sort_with.SortWith(*moveList, moves.Greater)
  util.RemoveDuplicates(moveList)
  return
}

