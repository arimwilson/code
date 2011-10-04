// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package scrabblish

import ("appengine"; "appengine/urlfetch"; "http"; "json";
        "scrabblish/scrabble"; "scrabblish/util")

type SolveRequest struct {
  Board [][]byte
  Tiles string
}

func init() {
  http.HandleFunc("/solve", solve)
}

func solve(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get our dictionary.
  client := urlfetch.Client(c)
  resp, err := client.Get("http://scrabblish.appspot.com/twl")
  if err != nil {
    c.Errorf("Could not retrieve twl with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  defer resp.Body.Close()
  dict := util.ReadWordList(resp.Body)

  // Get params from request.
  var solveRequest SolveRequest
  err = json.NewDecoder(r.Body).Decode(&solveRequest)
  if err != nil {
    c.Errorf("Could not decode request with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  tiles := util.ReadTiles(solveRequest.Tiles)
  letterValues := util.ReadLetterValues(
      "1 4 4 2 1 4 3 4 1 10 5 1 3 1 1 4 10 1 1 1 2 4 4 8 4 10")

  moveList := scrabble.GetMoveList(dict, solveRequest.Board, tiles,
                                   letterValues)

  w.Write([]byte(util.PrintMoveList(moveList, 25)))
}

