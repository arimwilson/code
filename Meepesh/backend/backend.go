package ariwilson

import (
  "appengine"; "appengine/datastore"; "appengine/user"; "encoding/json";
  "net/http"
)

func init() {
  http.HandleFunc("/backend/load", load)
  http.HandleFunc("/backend/save", save)
}

type World struct {
  User string
  Objects string
}

func load(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  query := datastore.NewQuery("world")
  query = query.Filter("User =", user.Current(c).String())
  world := new(World)
  _, err := query.Run(c).Next(world)
  if err != nil && err != datastore.Done {
    c.Errorf("Could not retrieve world for user %s with error: %s",
             user.Current(c).String(), err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  encoder := json.NewEncoder(w)
  encoder.Encode(world.Objects)
}

func save(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get params from request.
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  cur_user := user.Current(c).String()
  query := datastore.NewQuery("world")
  query.Filter("User =", cur_user)
  var key *datastore.Key
  world := new(World)
  key, err = query.Run(c).Next(world)
  if err == nil {
    world.Objects = r.FormValue("objects")
    _, err = datastore.Put(c, key, world)
  } else {
    world = &World{ cur_user, r.FormValue("objects") }
    _, err = datastore.Put(c, datastore.NewIncompleteKey(c, "world", nil),
                           world)
  }
  if err != nil {
    c.Errorf("Could not save world with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
}
