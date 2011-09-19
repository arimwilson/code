// Utility functions unrelated to core move generation for Scrabble.

package util

import ("bufio"; "container/vector"; "fmt"; "hash/crc32"; "strconv"; "strings";
        "os";
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

func Available(board [][]byte, location *moves.Location) bool {
  if location.X < 0 || location.X >= BOARD_SIZE || location.Y < 0 ||
     location.Y >= BOARD_SIZE {
    return false
  }
  letter := board[location.X][location.Y]
  return (letter < 'A' || letter > 'Z')
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
    tiles[tilesFlag[i]]++
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
  if (move.Direction != moves.ACROSS) { panic("Can't score down moves!") }
  wordMultiplier := 1
  score := 0
  for i := 0; i < len(move.Word); i++ {
    multiplier := board[move.Start.X][move.Start.Y + i]
    letterMultiplier := 1
    if multiplier == '1' || multiplier == '2' {
      wordMultiplier *= int(multiplier) - '0' + 1
    } else if multiplier == '3' || multiplier == '4' {
      letterMultiplier = int(multiplier) - '1'
    }
    score += letterMultiplier * letterValues[move.Word[i]]
  }
  move.Score += wordMultiplier * score
}

func PrintBoard(board [][]byte) {
  for i := 0; i < BOARD_SIZE; i++ {
    for j := 0; j < BOARD_SIZE; j++ {
      fmt.Printf("%c", board[i][j])
    }
    fmt.Printf("\n")
  }
}

func RemoveDuplicates(moveList *vector.Vector) {
  existingMoves := make(map[uint32] bool)
  for i := 0; i < moveList.Len(); i++ {
    move := moveList.At(i).(moves.Move)
    cksum := crc32.ChecksumIEEE(
      []byte(string([]int{move.Start.X, move.Start.Y, int(move.Direction)}) +
             move.Word))
    _, existing := existingMoves[cksum]
    if !existing {
      existingMoves[cksum] = true
    } else {
      moveList.Delete(i)
      i--
    }
  }
}

