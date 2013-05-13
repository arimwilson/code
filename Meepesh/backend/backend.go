package ariwilson

import (
  "net/http"
)

func init() {
  http.HandleFunc("/backend/load", load)
  http.HandleFunc("/backend/save", save)
}

func load(w http.ResponseWriter, r *http.Request) {
}

func save(w http.ResponseWriter, r *http.Request) {
}
