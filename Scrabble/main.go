// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.
//
// Sample usage:
// ./scrabblish -b empty_wordfeud.txt -t ABCDEFG

package main

import ("flag"; "fmt"; "log"; "runtime/pprof"; "os";
        "./scrabble"; "./util")

var wordListFlag = flag.String(
    "w", "twl.txt",
    "File with space-separated list of legal words, in upper-case.")
var boardFlag = flag.String(
    "b", "",
    "File with board structure. Format: * indicates starting point, 1 and 2 " +
    "indicate double and triple word score tiles, 3 and 4 indicate double " +
    "and triple letter score tiles, - indicates blank tiles, and upper-case " +
    "letters indicate existing words.")
var tilesFlag = flag.String(
    "t", "", "List of all 7 player tiles, in upper-case.")
var letterValuesFlag = flag.String(
    "l", "1 4 4 2 1 4 3 4 1 10 5 1 3 1 1 4 10 1 1 1 2 4 4 8 4 10",
    "Space-separated list of letter point values, from A-Z.")
var bonusFlag = flag.Int("o", 40, "Bonus for using all 7 tiles at once.")
var numResultsFlag = flag.Int(
    "n", 25, "Maximum number of results to output.")
var cpuProfileFlag = flag.String("c", "", "Write CPU profile to file.")

func main() {
  flag.Parse()
  if *cpuProfileFlag != "" {
    f, err := os.Create(*cpuProfileFlag)
    if err != nil {
      log.Fatal(err)
    }
    pprof.StartCPUProfile(f)
    defer pprof.StopCPUProfile()
  }

  wordListFile, err := os.Open(*wordListFlag);
  defer wordListFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -w, found %s\n", *wordListFlag)
    os.Exit(1)
  }
  boardFile, err := os.Open(*boardFlag);
  defer boardFile.Close();
  if err != nil {
    fmt.Printf("need valid file for -b, found %s\n", *boardFlag)
    os.Exit(1)
  }
  dict := util.ReadWordList(wordListFile)
  board := util.ReadBoard(boardFile)
  tiles := util.ReadTiles(*tilesFlag)
  letterValues := util.ReadLetterValues(*letterValuesFlag)

  moveList := scrabble.GetMoveList(dict, board, tiles, letterValues, *bonusFlag)

  util.PrintMoveList(moveList, board, *numResultsFlag)
}

