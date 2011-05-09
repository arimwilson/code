// Utility functions unrelated to core move generation for Scrabble.

package util

import ("bufio"; "fmt"; "strconv"; "strings"; "os";
        "./trie")

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

const BOARD_SIZE = 15

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

func ReadTiles(tilesFlag string) (tiles map[byte] bool) {
  tiles = make(map[byte] bool)
  for i := 0; i < len(tilesFlag); i++ {
    tiles[tilesFlag[i]] = true
  }
  return
}

func ReadLetterValues(letterValuesFlag string) (letterValues map[byte] int) {
  letterValues = make(map[byte] int)
  splitLetterValues := strings.Split(letterValuesFlag, " ", -1)
  for i := byte('A'); i <= byte('Z'); i++ {
    letterValues[i], _ = strconv.Atoi(splitLetterValues[i - 'A'])
  }
  return
}

func PrintBoard(board [][]byte) {
  for i := 0; i < BOARD_SIZE; i++ {
    for j := 0; j < BOARD_SIZE; j++ {
      fmt.Printf("%c", board[i][j])
    }
    fmt.Printf("\n")
  }
}

