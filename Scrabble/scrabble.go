// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package main

import ("bufio"; "flag"; "fmt"; "os";)

var wordList = flag.String("w", "",
                           "File with space-separated list of legal words.")
var board = flag.String("b", "", "File with board structure.")
var tiles = flag.String("t", "", "Comma-separated list of player tiles.")

func readWordList(wordListFile* os.File) {
  wordListReader := bufio.NewReader(wordListFile)
  for {
    word, err := wordListReader.ReadString(" "[0])
    if err != nil {
      return
    }
    fmt.Printf(word + "\n")
  }
}

func readBoard(boardFile* os.File) {
  
}

func main() {
  flag.Parse()
  wordListFile, err := os.Open(*wordList, os.O_RDONLY, 0);
  defer wordListFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -w, found %s", *wordList)
    os.Exit(1)
  }
  readWordList(wordListFile)
  boardFile, err := os.Open(*board, os.O_RDONLY, 0);
  defer boardFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -b, found %s", *board)
    os.Exit(1)
  }
  readBoard(boardFile)
}
