package scrabble

import ("container/vector";
        "cross_check"; "moves"; "trie"; "sort_with"; "util")

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

// Skip over existing tiles and continue looking for words to the right.
func MoveRight(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, crossChecks map[int] map[byte] int,
    possibleMove moves.Move) (moveList *vector.Vector) {
  moveList = new(vector.Vector)
  prefixEnd := moves.Location{possibleMove.Start.X,
                              possibleMove.Start.Y + len(possibleMove.Word)}
  for ; prefixEnd.Y < util.BOARD_SIZE && !util.Available(board, &prefixEnd);
      prefixEnd.Y++ {
    possibleMove.Word += string(board[prefixEnd.X][prefixEnd.Y])
  }
  if CanFollow(dict, possibleMove.Word, tiles) {
    moveList.AppendVector(
      Extend(dict, board, tiles, letterValues, crossChecks, possibleMove,
             false))
  }
  return
}

// Add one letter to a possible move, checking if we've got a word, going either
// left or right.
func Extend(
    dict *trie.Trie, board [][]byte, tiles map[byte] int,
    letterValues map[byte] int, crossChecks map[int] map[byte] int,
    possibleMove moves.Move, left bool) (moveList *vector.Vector) {
  moveList = new(vector.Vector)
  var positionCrossChecks map[byte] int
  var existing bool
  if left {
    if !util.Available(board, &possibleMove.Start) { return }
    positionCrossChecks, existing = crossChecks[possibleMove.Start.Hash()]
  } else {
    endLocation := moves.Location{possibleMove.Start.X,
                                  possibleMove.Start.Y + len(possibleMove.Word)}
    if !util.Available(board, &endLocation) { return }
    positionCrossChecks, existing = crossChecks[endLocation.Hash()]
  }
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
        var placedCol int
        if left {
          placedCol = possibleMove.Start.Y
        } else {
          placedCol = possibleMove.Start.Y + len(possibleMove.Word)
        }
        verticallyScoredLetters[i - 26] = BlankScore(
            score, letterValues[i], board[possibleMove.Start.X][placedCol])
      }
    } else {
      for i := 'A'; i <= 'Z'; i++ {
        verticallyScoredLetters[byte(i - 26)] = 0
      }
    }
    for letter, score := range(verticallyScoredLetters) {
      placedMove := possibleMove.Copy()
      if left {
        placedMove.Word = string(letter) + placedMove.Word
      } else {
        placedMove.Word += string(letter)
      }
      placedMove.Score += score
      if dict.Find(placedMove.Word) {
        score = placedMove.Score
        util.Score(board, letterValues, &placedMove)
        moveList.Push(placedMove)
        placedMove.Score = score
      }
      tiles[tile]--
      moveList.AppendVector(
        MoveRight(dict, board, tiles, letterValues, crossChecks, placedMove))
      if left {
        placedMove.Start.Y--
        moveList.AppendVector(
          Extend(dict, board, tiles, letterValues, crossChecks, placedMove,
                 true))
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
    letterValues map[byte] int,
    crossChecks map[int] map[byte] int) (moveList *vector.Vector) {
  moveList = new(vector.Vector)
  for i := 0; i < util.BOARD_SIZE; i++ {
    for j := 0; j < util.BOARD_SIZE; j++ {
      possibleMove := moves.Move{
        Word: "", Score: 0, Start: moves.Location{i, j},
        Direction: moves.ACROSS }
      if board[i][j] == '*' {
        moveList.AppendVector(Extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            true))
      } else if board[i][j] >= 'A' && board[i][j] <= 'Z' {
        possibleMove.Start.Y--
        possibleMove.Word = string(board[i][j])
        moveList.AppendVector(Extend(
            dict, board, tiles, letterValues, crossChecks,
            possibleMove, true))
        possibleMove.Start.Y++
        moveList.AppendVector(MoveRight(
            dict, board, tiles, letterValues, crossChecks, possibleMove))
        possibleMove.Word = ""
        possibleMove.Start.X--
        moveList.AppendVector(Extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            true))
        possibleMove.Start.X += 2
        moveList.AppendVector(Extend(
            dict, board, tiles, letterValues, crossChecks, possibleMove,
            true))
      }
    }
  }
  return
}

// Set Direction for all moves in moveList to direction.
func SetDirection(direction moves.Direction, moveList *vector.Vector) {
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    if (move.Direction != direction) {
      move.Start.X, move.Start.Y = move.Start.Y, move.Start.X
    }
    move.Direction = direction
    moveList.Set(i, move)
  }
}

// Get all possible moves on board, ordered by score, given params.
func GetMoveList(dict *trie.Trie, board [][]byte, tiles map[byte] int,
                 letterValues map[byte] int) (moveList *vector.Vector) {
  transposedBoard := util.Transpose(board)
  crossChecks := cross_check.GetCrossChecks(dict, transposedBoard, tiles,
                                            letterValues)
  moveList = GetMoveListAcross(dict, board, tiles, letterValues, crossChecks)
  SetDirection(moves.ACROSS, moveList)
  downCrossChecks := cross_check.GetCrossChecks(dict, board, tiles,
                                                letterValues)
  downMoveList := GetMoveListAcross(
      dict, transposedBoard, tiles, letterValues, downCrossChecks)
  SetDirection(moves.DOWN, downMoveList)
  moveList.AppendVector(downMoveList)
  sort_with.SortWith(*moveList, moves.Greater)
  util.RemoveDuplicates(moveList)
  return
}

