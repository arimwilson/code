// Scrabble move generator. Given a word list, board, and your current tiles,
// outputs all legal moves ranked by point value.

package scrabblish

import ("appengine"; "appengine/datastore"; "appengine/memcache";
        "appengine/urlfetch"; "appengine/user"; "bytes"; "encoding/binary";
        "fmt"; "gob"; "http"; "json"; "strconv";
        "scrabblish/scrabble"; "scrabblish/trie"; "scrabblish/util")

func init() {
  http.HandleFunc("/save", save)
  http.HandleFunc("/list", list)
  http.HandleFunc("/solve", solve)
}

type Board struct {
  User string
  Name string
  Board string
}

func save(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get params from request.
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  cur_user := user.Current(c).String()
  name := r.FormValue("name")
  query := datastore.NewQuery("board")
  query.Filter("User =", user.Current(c).String())
  query.Filter("Name = ", name)
  var key *datastore.Key
  board := new(Board)
  key, err = query.Run(c).Next(board)
  if err == nil {
    board.Board = r.FormValue("board")
    _, err = datastore.Put(c, key, board)
  } else {
    board = &Board{ cur_user, name, r.FormValue("board") }
    _, err = datastore.Put(c, datastore.NewIncompleteKey(c, "board", nil),
                           board)
  }
  if err != nil {
    c.Errorf("Could not save board with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
}

func list(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  query := datastore.NewQuery("board")
  query.Filter("User =", user.Current(c).String())
  results := new([]Board)
  _, err := query.GetAll(c, results)
  if err != nil {
    c.Errorf("Could not retrieve boards for user %s with error: %s",
             user.Current(c).String(), err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
  encoder := json.NewEncoder(w)
  encoder.Encode(results)
}

func bToI(b []byte) int32 {
  buf := bytes.NewBuffer(b)
  var i int32
  binary.Read(buf, binary.LittleEndian, &i)
  return i
}

func iToB(i int32) []byte {
  b := make([]byte, 4)
  for j := 0; j < 4; j++ {
    b[j] = byte(i >> uint(8 * j))
  }
  return b
}

// Get the list of subkeys corresponding to the primary key.
func getKeys(c appengine.Context, key string) (keys []string) {
  item, err := memcache.Get(c, key);
  if err != nil {
    c.Infof("Could not retrieve number of keys with error: %s", err.String())
    return
  }
  num := bToI(item.Value)
  keys = make([]string, num)
  for i := int32(0); i < num; i++ {
    keys[i] = fmt.Sprintf("%s%d", key, i)
  }
  return
}

const MAX_MEMCACHE_VALUE_SIZE = 1000000

// Split a byte stream into memcache items of fixed size, with given key and
// subkeys.
func splitMemcache(key string, data []byte) (items []*memcache.Item) {
  item := new(memcache.Item)
  item.Key = key
  item.Value = iToB(int32((len(data) - 1) / MAX_MEMCACHE_VALUE_SIZE + 1))
  items = append(items, item)
  for i := 0; i < len(data); i += MAX_MEMCACHE_VALUE_SIZE {
    item = new(memcache.Item)
    item.Key = fmt.Sprintf("%s%d", key, i / MAX_MEMCACHE_VALUE_SIZE)
    j := i + MAX_MEMCACHE_VALUE_SIZE
    if j > len(data) { j = len(data) }
    item.Value = data[i:j]
    items = append(items, item)
  }
  return
}

// Join a byte stream, in order, from given subkeys.
func joinMemcache(keys []string,
                  items map[string]*memcache.Item) (data []byte) {
  for i := 0; i < len(keys); i++ {
    item, existing := items[keys[i]]
    if !existing {
      panic(fmt.Sprintf("No item for key %s!", keys[i]))
    }
    data = append(data, item.Value...)
  }
  return
}

func solve(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  var dict *trie.Trie
  // Get our dictionary.
  keys := getKeys(c, "dict")
  items, err := memcache.GetMulti(c, keys)
  if len(keys) == 0 || err != nil || len(keys) != len(items) {
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
    err = enc.Encode(*dict)
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
    data := bytes.NewBuffer(joinMemcache(keys, items))
    dec := gob.NewDecoder(data)
    dict = new(trie.Trie)
    err := dec.Decode(dict)
    if err != nil {
      c.Errorf("Could not decode dict with error: %s", err.String())
      http.Error(w, err.String(), http.StatusInternalServerError)
      return
    }
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
  letterValues := util.ReadLetterValues(r.FormValue("letterValues"))
  bonus, _ := strconv.Atoi(r.FormValue("bonus"))

  moveList := scrabble.GetMoveList(dict, board, tiles, letterValues, bonus)

  fmt.Fprint(w, util.PrintMoveList(moveList, 25))
}

