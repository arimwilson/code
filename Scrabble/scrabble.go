// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package main

import ("bufio"; "flag"; "fmt"; "os"; "strings"; "./trie")

var wordListFlag = flag.String("w", "",
                               "File with space-separated list of legal words.")
var boardFlag = flag.String("b", "", "File with board structure.")
var tilesFlag = flag.String("t", "", "Comma-separated list of player tiles.")

type Direction int
const {
  RIGHT = iota
  DOWN
}

type Location struct {
  x int
  y int
}

type Move struct {
  word string
  score int
  start Location
  direction Direction
}

func readWordList(wordListFile* os.File) (dict *trie.Trie) {
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
  return
}

func getMoveList(dict *trie.Trie, board [15[15]byte) (moves []Move) {
  return
}

func main() {
  flag.Parse()
  wordListFile, err := os.Open(*wordListFlag, os.O_RDONLY, 0);
  defer wordListFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -w, found %s\n", *wordListFlag)
    os.Exit(1)
  }
  boardFile, err := os.Open(*boardFlag, os.O_RDONLY, 0);
  defer boardFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -b, found %s\n", *boardFlag)
    os.Exit(1)
  }
  tiles := strings.Split(*tilesFlag, ",", -1)
  if len(tiles) != 7 {
    fmt.Printf("need 7 tiles in -t, found %d\n", len(tiles))
    os.Exit(1)
  }
  dict := readWordList(wordListFile)
  board := readBoard(boardFile)
  getMoveList(dict, board)
  // TODO(ariw): Sort by score, print out move list.
}

