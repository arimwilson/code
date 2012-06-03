package ariwilson

import (
  "fmt"
  "net/http"
)

func init() {
    http.HandleFunc("/", handler)
}

func handler(w http.ResponseWriter, r *http.Request) {
  fmt.Fprint(w, "You have reached the secret backdoor!")
}
