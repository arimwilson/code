package cross_check_test

import ("container/vector"; "fmt"; "testing";
        "cross_check"; "moves"; "trie"; "util")

func TestGetCrossChecks(t *testing.T) {
  util.BOARD_SIZE = 5
  dict := trie.New()
  dict.Insert("abra")
  dict.Insert("boo")
  dict.Insert("chit")
  dict.Insert("ab")

  transposedBoard := [][]byte{
      []byte{"--a--"},
      []byte{"--b--"},
      []byte{"--r--"},
      []byte{"--a--"},
      []byte{"-----"}}

  tiles := make(map[byte] int)
  tiles['a'] = 1
  tiles['b'] = 1
  tiles['r'] = 1
  tiles['a'] = 1

  letterValues := copy(tiles)

  crossChecks := cross_check.GetCrossChecks(
      dict, transposedBoard, tiles, letterValues)
  for k, v := range crossChecks {
    for i := 0; i < v.Len(); ++i {
      fmt.Printf("CrossCheck: %c", v.At(i).(CrossCheck).letter)
      // TODO(ariw): Assert about this.
    }
  }
}

