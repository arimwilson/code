// Utility functions unrelated to core move generation for Scrabble.

package util

import ("bufio"; "container/vector"; "fmt"; "io"; "strconv"; "strings";
        "scrabblish/moves"; "scrabblish/trie")

var BOARD_SIZE = 15

func Existing(board [][]byte, location *moves.Location) bool {
  if location.X < 0 || location.X >= BOARD_SIZE || location.Y < 0 ||
     location.Y >= BOARD_SIZE {
    return false
  }
  tile := board[location.X][location.Y]
  return tile >= 'A' && tile <= 'Z'
}

func Available(board [][]byte, location *moves.Location) bool {
  if location.X < 0 || location.X >= BOARD_SIZE || location.Y < 0 ||
     location.Y >= BOARD_SIZE {
    return false
  }
  tile := board[location.X][location.Y]
  return tile < 'A' || tile > 'Z'
}

func ReadWordList(wordList io.Reader) (dict* trie.Trie) {
  wordListReader := bufio.NewReader(wordList)
  dict = trie.New()
  for {
    word, err := wordListReader.ReadString(' ')
    if err != nil {
      return
    }
    dict.Insert(strings.TrimSpace(word))
  }
  return
}

func ReadBoard(boardFlat string) (board [][]byte) {
  board = make([][]byte, BOARD_SIZE)
  for i := 0; i < BOARD_SIZE; i++ {
    board[i] = make([]byte, BOARD_SIZE)
    board[i] = []byte(boardFlat[BOARD_SIZE * i:BOARD_SIZE * (i + 1)])
  }
  return
}

func ReadTiles(tilesFlag string) (tiles map[byte] int) {
  tiles = make(map[byte] int)
  for i := 0; i < len(tilesFlag); i++ {
    tiles[tilesFlag[i]]++
  }
  return
}

func ReadLetterValues(letterValuesFlag string) (letterValues map[byte] int) {
  letterValues = make(map[byte] int)
  splitLetterValues := strings.Split(letterValuesFlag, " ", -1)
  for i := 'A'; i <= 'Z'; i++ {
    letterValues[byte(i)], _ = strconv.Atoi(splitLetterValues[i - 'A'])
  }
  return
}

func Transpose(board [][]byte) (transposedBoard [][]byte) {
  transposedBoard = make([][]byte, BOARD_SIZE)
  for i := 0; i < BOARD_SIZE; i++ {
    transposedBoard[i] = make([]byte, BOARD_SIZE)
    copy(transposedBoard[i], board[i])
  }
  for i := 0; i < BOARD_SIZE; i++ {
    for j := 0; j < i; j++ {
      transposedBoard[i][j], transposedBoard[j][i] =
          transposedBoard[j][i], transposedBoard[i][j]
    }
  }
  return
}

func TileMultipliers(tile *byte) (wordMultiplier int, letterMultiplier int) {
  wordMultiplier = 1
  letterMultiplier = 1
  if *tile == '1' || *tile == '2' {
    wordMultiplier *= int(*tile - '0' + 1)
  } else if *tile == '3' || *tile == '4' {
    letterMultiplier = int(*tile - '1')
  }
  return
}

func Score(board [][]byte, letterValues map[byte] int, move *moves.Move) {
  // We ensure that the move is going right, for cache friendliness.
  if (move.Direction != moves.ACROSS) { panic("Can't score down moves!") }
  wordMultiplier := 1
  score := 0
  boardTilesUsed := 0
  for i := 0; i < len(move.Word); i++ {
    tile := board[move.Start.X][move.Start.Y + i]
    if tile >= 'A' && tile <= 'Z' { boardTilesUsed++ }
    tileWordMultiplier, letterMultiplier := TileMultipliers(&tile)
    wordMultiplier *= tileWordMultiplier
    // TODO(ariw): Remove hack.
    if move.Word[i] >= 'A' {
      score += letterMultiplier * letterValues[move.Word[i]]
    }
  }
  move.Score += wordMultiplier * score
  // Scrabble!
  if len(move.Word) - boardTilesUsed == 7 {
    move.Score += 40
  }
}

func RemoveDuplicates(moveList *vector.Vector) {
  existingMoves := make(map[uint32] bool)
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    hash := move.Hash()
    _, existing := existingMoves[hash]
    if !existing {
      existingMoves[hash] = true
    } else {
      moveList.Delete(i)
      i--
    }
  }
}

func PrintBoard(board [][]byte) {
  for i := 0; i < BOARD_SIZE; i++ {
    for j := 0; j < BOARD_SIZE; j++ {
      fmt.Printf("%c", board[i][j])
    }
    fmt.Printf("\n")
  }
}

func PrintMoveOnBoard(board [][]byte, move *moves.Move) {
  word := moves.MoveWord(move)
  for i := 0; i < BOARD_SIZE; i++ {
    for j := 0; j < BOARD_SIZE; j++ {
      if move.Direction == moves.ACROSS && i == move.Start.X &&
         j >= move.Start.Y && j < move.Start.Y + len(word) {
        fmt.Printf("%c", word[j - move.Start.Y])
      } else if move.Direction == moves.DOWN && j == move.Start.Y &&
                i >= move.Start.X && i < move.Start.X + len(word) {
        fmt.Printf("%c", word[i - move.Start.X])
      } else {
        fmt.Printf("%c", board[i][j])
      }
    }
    fmt.Printf("\n")
  }
}

func PrintMoveList(moveList *vector.Vector, numResults int) string {
  numMoves := numResults
  if numResults <= 0 || moveList.Len() < numResults {
    numMoves = moveList.Len()
  }
  eachMove := make([]string, numMoves)
  for i := 0; i < numMoves; i++ {
    move := moveList.At(i).(moves.Move)
    eachMove[i] = fmt.Sprintf("%d. %s", i + 1, moves.PrintMove(&move))
  }
  return strings.Join(eachMove, "<br>")
}

func TestInsertIntoDictionary() (dict *trie.Trie) {
  dict = trie.New()
  var strings = []string{
      "ABBA",
      "ABRA",
      "EXISTING",
      "TEXTUAL",
      "LATER"}
  for i := 0; i < len(strings); i++ {
    dict.Insert(strings[i])
  }
  return
}

