package sort_with

import ("sort")

type sortWrap struct {
  sl []interface{}
  fn func(interface{}, interface{}) bool
}

func (self sortWrap) Len() int {
  return len(self.sl)
}

func (self sortWrap) Swap(i, j int) {
  s := self.sl
  s[i], s[j] = s[j], s[i]
}

func (self sortWrap) Less(i, j int) bool {
  return self.fn(self.sl[i], self.sl[j])
}

type LessFn func(interface{}, interface{}) bool

func SortWith(vector []interface{}, fn LessFn) {
    sort.Sort(sortWrap{vector, fn})
}

