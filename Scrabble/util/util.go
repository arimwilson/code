// Utility functions unrelated to core move generation for Scrabble.

package util

import ("bufio"; "fmt"; "strconv"; "strings"; "os";
        "moves"; "trie")

func ReadWordList(wordListFile* os.File) (dict* trie.Trie) {
  wordListReader := bufio.NewReader(wordListFile)
  dict = trie.New()
  for {
    word, err := wordListReader.ReadString(" "[0])
    if err != nil {
      return
    }
    dict.Insert(strings.TrimSpace(word))
  }
  return
}

var BOARD_SIZE = 15

func Existing(board [][]byte, location *moves.Location) bool {
  if location.X < 0 || location.X > BOARD_SIZE || location.Y < 0 ||
     location.Y > BOARD_SIZE {
    return false
  }
  char := board[location.X][location.Y]
  return (char >= 'A' && char <= 'Z') || char == '*'
}

func ReadBoard(boardFile* os.File) (board [][]byte) {
  board = make([][]byte, BOARD_SIZE)
  for i := 0; i < BOARD_SIZE; i++ {
    board[i] = make([]byte, BOARD_SIZE)
    _, err := boardFile.Read(board[i])
    if err != nil {
      os.Exit(1)
    }
    _, err = boardFile.Seek(1, 1)
    if err != nil {
      os.Exit(1)
    }
  }
  return
}

func ReadTiles(tilesFlag string) (tiles map[byte] int) {
  tiles = make(map[byte] int)
  for i := 0; i < len(tilesFlag); i++ {
    tile, ok := tiles[tilesFlag[i]]
    if !ok {
      tile = 0
    } else {
      tile++
    }
  }
  return
}

func ReadLetterValues(letterValuesFlag string) (letterValues map[byte] int) {
  letterValues = make(map[byte] int)
  splitLetterValues := strings.Split(letterValuesFlag, " ")
  for i := byte('A'); i <= byte('Z'); i++ {
    letterValues[i], _ = strconv.Atoi(splitLetterValues[i - 'A'])
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

func Score(board [][]byte, letterValues map[byte] int, move *moves.Move) {
  // We ensure that the move is going right, for cache friendliness.
  if (move.Direction != moves.RIGHT) { panic("Can't score down moves!") }
  wordMultiplier := 1
  move.Score = 0
  for i := 0; i < len(move.Word); i++ {
    multiplier := board[move.Start.X][move.Start.Y + i]
    letterMultiplier := 1
    if multiplier == '1' || multiplier == '2' {
      wordMultiplier *= int(multiplier) - '0' + 1
    } else if multiplier == '3' || multiplier == '4' {
      letterMultiplier = int(multiplier) - '1'
    }
    move.Score += letterMultiplier * letterValues[move.Word[i]]
  }
  move.Score *= wordMultiplier
}

func PrintBoard(board [][]byte) {
  for i := 0; i < BOARD_SIZE; i++ {
    for j := 0; j < BOARD_SIZE; j++ {
      fmt.Printf("%c", board[i][j])
    }
    fmt.Printf("\n")
  }
}

