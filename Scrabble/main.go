// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package main

import ("flag"; "fmt"; "os";
        "cross_check"; "moves"; "scrabble"; "sort_with"; "util")

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
var numResultsFlag = flag.Int(
    "n", 25, "Maximum number of results to output.")

func main() {
  flag.Parse()
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
  if len(*tilesFlag) != 7 {
    fmt.Printf("need 7 tiles in -t, found %d\n", len(*tilesFlag))
    os.Exit(1)
  }
  dict := util.ReadWordList(wordListFile)
  board := util.ReadBoard(boardFile)
  tiles := util.ReadTiles(*tilesFlag)
  letterValues := util.ReadLetterValues(*letterValuesFlag)
  transposedBoard := util.Transpose(board)

  // Get moves going both right and down.
  crossChecks := cross_check.GetCrossChecks(dict, transposedBoard, tiles,
                                            letterValues)
  moveList := scrabble.GetMoveList(dict, board, tiles, letterValues,
                                   crossChecks)
  scrabble.SetDirection(moves.ACROSS, moveList)
  downCrossChecks := cross_check.GetCrossChecks(dict, board, tiles,
                                                letterValues)
  downMoveList := scrabble.GetMoveList(dict, transposedBoard, tiles,
                                       letterValues, downCrossChecks)
  scrabble.SetDirection(moves.DOWN, downMoveList)
  moveList.AppendVector(downMoveList)
  sort_with.SortWith(*moveList, moves.Greater)
  util.RemoveDuplicates(moveList)
  for i := 0;
      (*numResultsFlag <= 0 || i < *numResultsFlag) && i < moveList.Len(); i++ {
    fmt.Printf("%d. ", i + 1)
    move := moveList.At(i).(moves.Move)
    moves.PrintMove(&move)
    util.PrintMoveOnBoard(board, &move)
  }
}

