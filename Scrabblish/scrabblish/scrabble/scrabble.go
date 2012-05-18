package scrabble

import ("appengine"; "appengine/user"; "sort";
        "scrabblish/cross_check"; "scrabblish/moves"; "scrabblish/trie";
        "scrabblish/util")

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

var iter_count = 0

// Add one letter to a possible move, checking if we've got a word, going either
// left or right.
func Extend(
    c appengine.Context, dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, bonus int, crossChecks map[int] map[byte] int,
    possibleMove moves.Move, left bool) (moveList []moves.Move) {
  // TODO(ariw): Remove this nonsense once a work-around of Go AppEngine's
  // single-threadedness is found.
  if iter_count % 10000 == 0 {
    _, _ = user.LoginURL(c, "test")
  }
  iter_count++
  moveList = make([]moves.Move, 0)
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
        moveList = append(moveList, placedMove)
        placedMove.Score = score
      }
      tiles[tile]--
      if CanFollow(dict, placedMove.Word, tiles) {
        moveList = append(
            moveList,
            Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                   placedMove, false)...)
      }
      if left {
        placedMove.Start.Y--
        moveList = append(
            moveList,
            Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                   placedMove, true)...)
      }
      tiles[tile]++
    }
  }
  return
}

// Look for new across moves connected to any existing tile. Duplicates are
// possible.
func GetMoveListAcross(
    c appengine.Context, dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, bonus int,
    crossChecks map[int] map[byte] int) (moveList []moves.Move) {
  moveList = make([]moves.Move, 0)
  possibleMove := moves.Move{ Word: "", Score: 0, Direction: moves.ACROSS }
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      if board[i][j] == '*' {
        possibleMove.Start = moves.Location{ i, j }
        moveList = append(
            moveList,
            Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                   possibleMove, true)...)
      } else if board[i][j] >= 'A' && board[i][j] <= 'Z' {
        possibleMove.Start.X = i
        possibleMove.Start.Y = j - 1
        possibleMove.Word = GetExistingRightTiles(board, &possibleMove.Start)
        moveList = append(
            moveList,
            Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                   possibleMove, true)...)
        possibleMove.Start.Y = j + 1
        possibleMove.Word = GetExistingLeftTiles(board, &possibleMove.Start)
        moveList = append(
            moveList,
            Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                   possibleMove, false)...)
        possibleMove.Start.Y = j
        possibleMove.Word = ""
        leftUp := moves.Location{i - 1, j - 1}
        rightUp := moves.Location{i - 1, j + 1}
        if !util.Existing(board, &leftUp) && !util.Existing(board, &rightUp) {
          possibleMove.Start.X = i - 1
          moveList = append(
              moveList,
              Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                     possibleMove, true)...)
        }
        leftDown := moves.Location{i + 1, j - 1}
        rightDown := moves.Location{i + 1, j + 1}
        if !util.Existing(board, &leftDown) &&
           !util.Existing(board, &rightDown) {
          possibleMove.Start.X = i + 1
          moveList = append(
              moveList,
              Extend(c, dict, board, tiles, letterValues, bonus, crossChecks,
                     possibleMove, true)...)
        }
      }
    }
  }
  return
}

// Set Direction for all moves in moveList to direction.
func SetDirection(direction moves.Direction, moveList []moves.Move) {
  for i := 0; i < len(moveList); i++ {
    move := moveList[i]
    if move.Direction != direction {
      move.Start.X, move.Start.Y = move.Start.Y, move.Start.X
    }
    move.Direction = direction
    moveList[i] = move
  }
}

// Get all possible moves on board, ordered by score, given params.
func GetMoveList(
    c appengine.Context, dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, bonus int) (moveList []moves.Move) {
  transposedBoard := util.Transpose(board)
  crossChecks := cross_check.GetCrossChecks(dict, transposedBoard, letterValues)
  moveList = GetMoveListAcross(c, dict, board, tiles, letterValues, bonus,
                               crossChecks)
  SetDirection(moves.ACROSS, moveList)
  downCrossChecks := cross_check.GetCrossChecks(dict, board, letterValues)
  downMoveList := GetMoveListAcross(
      c, dict, transposedBoard, tiles, letterValues, bonus, downCrossChecks)
  SetDirection(moves.DOWN, downMoveList)
  moveList = append(moveList, downMoveList...)
  sort.Sort(moves.Moves(moveList))
  util.RemoveDuplicates(moveList)
  return
}

