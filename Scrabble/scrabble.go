package scrabble

import ("flag"; "fmt"; "os";)

var wordList = flag.String("w", "",
                           "File with space-separated list of legal words.")
var board = flag.String("b", "", "File with board structure.")

func main() {
  flag.Parse()
  wordListFile = os.Open(wordList, os.O_READONLY, 0);
  for i := 0; 
  boardFile = os.Open(board, os.O_READONLY, 0);
  os.Stdin
}
