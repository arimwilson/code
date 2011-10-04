// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package scrabblish

import ("appengine"; "appengine/urlfetch"; "http"; "json";
        "scrabble"; "util")

type SolveRequest struct {
  Board [][]byte
  Tiles string
}

func init() {
  http.HandleFunc("/solve", solve)
}

func solve(w http.ResponseWriter, r *http.Request) {
  // Get our dictionary.
  c := appengine.NewContext(r)
  client := urlfetch.Client(c)
  resp, err := client.Get("twl")
  if err != nil {
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  defer resp.body.close()
  dict := util.ReadWordList(&resp.Body)

  // Get params from request.
  var solveRequest SolveRequest
  err := json.NewDecoder(r.Body).Unmarshal(&solveRequest)
  if err != nil {
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  letterValues := util.ReadLetterValues(
      "1 4 4 2 1 4 3 4 1 10 5 1 3 1 1 4 10 1 1 1 2 4 4 8 4 10")

  moveList := scrabble.GetMoveList(dict, solveRequest.Board, solveRequest.Tiles,
                                   letterValues)

  w.Write([]byte(PrintMoveList(&moveList, 25)))
}

