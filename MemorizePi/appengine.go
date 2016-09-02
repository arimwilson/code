// 

package appengine

import (
  "appengine";
  "fmt";
  "net/http";
)

func init() {
  http.HandleFunc("/test", test)
}

func test(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get params from request.
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  w.Write([]byte("moo"))
}
