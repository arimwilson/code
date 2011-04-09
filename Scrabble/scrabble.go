// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package main

import ("bufio"; "flag"; "fmt"; "os"; "strings")

var wordListFlag = flag.String("w", "",
                               "File with space-separated list of legal words.")
var boardFlag = flag.String("b", "", "File with board structure.")
var tilesFlag = flag.String("t", "", "Comma-separated list of player tiles.")

/*
type Trie struct {
  Terminal bool
  Children Trie[]
}
*/

func readWordList(wordListFile* os.File) {
  wordListReader := bufio.NewReader(wordListFile)
  for {
    _, err := wordListReader.ReadString(" "[0])
    if err != nil {
      return
    }
    // TODO(ariw): Read into trie.
  }
}

func readBoard(boardFile* os.File) (board [15][15]byte) {
  for i := 0; i < 15; i++ {
    _, err := boardFile.Read(board[i][:])
    if err != nil {
      os.Exit(1)
    }
    _, err = boardFile.Seek(1, 1)
    if err != nil {
      os.Exit(1)
    }
  }
  return board
}

func main() {
  flag.Parse()
  wordListFile, err := os.Open(*wordListFlag, os.O_RDONLY, 0);
  defer wordListFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -w, found %s", *wordListFlag)
    os.Exit(1)
  }
  readWordList(wordListFile)
  boardFile, err := os.Open(*boardFlag, os.O_RDONLY, 0);
  defer boardFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -b, found %s", *boardFlag)
    os.Exit(1)
  }
  readBoard(boardFile)
  strings.Split(*tilesFlag, ",", -1)
}

