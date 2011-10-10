// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package scrabblish

import ("appengine"; "appengine/memcache"; "appengine/urlfetch"; "bytes"; "fmt";
        "gob"; "http";
        "scrabblish/scrabble"; "scrabblish/trie"; "scrabblish/util")

func init() {
  http.HandleFunc("/solve", solve)
}

const MAX_MEMCACHE_VALUE_SIZE = 1048576

func getKeys(key string, num int) (keys []string) {
  keys = make([]string, num)
  for i := 0; i < num; i++ {
    keys[i] = fmt.Sprintf("%s%d", key, i)
  }
  return
}

func splitMemcache(key string, data []byte) (items []*memcache.Item) {
  keys := getKeys(key, len(data))
  for i := 0; i < len(data); i += MAX_MEMCACHE_VALUE_SIZE {
    item := new(memcache.Item)
    item.Key = keys[i]
    j := i + MAX_MEMCACHE_VALUE_SIZE
    if j > len(data) { j = len(data) }
    item.Value = data[i:j]
  }
  return
}

func joinMemcache(items map[string]*memcache.Item) (data []byte) {
  for _, value := range(items) {
    data = append(data, value.Value...)
  }
  return
}

func solve(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  var dict *trie.Trie
  // Get our dictionary.
  items, err := memcache.GetMulti(c, getKeys("dict", 2))
  if err != nil || len(items) == 0 {
    client := urlfetch.Client(c)
    resp, err := client.Get("http://scrabblish.appspot.com/twl")
    if err != nil {
      c.Errorf("Could not retrieve twl with error: %s", err.String())
      http.Error(w, err.String(), http.StatusInternalServerError)
      return
    }
    defer resp.Body.Close()
    dict = util.ReadWordList(resp.Body)
    var data bytes.Buffer
    enc := gob.NewEncoder(&data)
    err = enc.Encode(dict)
    if err != nil {
      c.Errorf("Could not encode twl with error: %s", err.String())
    }
    errs := memcache.SetMulti(c, splitMemcache("dict", data.Bytes()))
    for i := 0; i < len(errs); i++ {
      if errs[i] != nil {
        c.Errorf("Could not cache dict chunk %d with error: %s", i,
                 errs[i].String())
      }
    }
  } else {
    data := bytes.NewBuffer(joinMemcache(items))
    dec := gob.NewDecoder(data)
    dec.Decode(dict)
  }

  // Get params from request.
  err = r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  board := util.ReadBoard(r.FormValue("board"))
  tiles := util.ReadTiles(r.FormValue("tiles"))
  letterValues := util.ReadLetterValues(
      "1 4 4 2 1 4 3 4 1 10 5 1 3 1 1 4 10 1 1 1 2 4 4 8 4 10")

  moveList := scrabble.GetMoveList(dict, board, tiles,
                                   letterValues)

  fmt.Fprint(w, util.PrintMoveList(moveList, 25))
}

