package meepesh

import (
  "appengine"; "appengine/datastore"; "appengine/user"; "bytes";
  "compress/gzip"; "encoding/json"; i"ioutil"; "net/http"
)

func init() {
  http.HandleFunc("/backend/load", load)
  http.HandleFunc("/backend/save", save)
}

type World struct {
  User string
  Name string
  Version int
  Data []byte
}

func getWorld(c appengine.Context, cur_user string, name string) (
    *datastore.Key, *World, error) {
  query := datastore.NewQuery("world")
  query.Filter("User =", cur_user)
  query.Filter("Name =", name)
  world := new(World)
  key, err := query.Run(c).Next(world)
  return key, world, err
}

func unzip([]byte bytes) ([]byte, error) {
  buffer := bytes.NewBuffer(bytes)
  reader, err := gzip.NewReader(buffer)
  if err != nil {
    return nil, err
  }
  var unzipped_bytes []byte
  unzipped_bytes, err = ioutil.ReadAll(reader)
  return unzipped_bytes, err
}

func load(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get params from request.
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  cur_user := user.Current(c).String()
  name := r.FormValue("name")
  var world *World
  _, world, err = getWorld(c, cur_user, r.FormValue("name"))
  if err != nil && err != datastore.Done {
    c.Errorf("Could not load world %s for user %s with error: %s",
             name, cur_user, err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  encoder := json.NewEncoder(w)
  var data []byte
  data, err = unzip(world.Data)
  if err != nil {
    c.Errorf("Could not decompress data with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  encoder.Encode(string(data))
}

func zip([]byte bytes) ([]byte, error) {
  buffer := new(bytes.Buffer)
  writer := gzip.NewWriter(buffer)
  _, err := writer.Write(bytes)
  if err != nil {
    return nil, err
  }
  err = writer.Close()
  return buffer.Bytes(), err
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
  name := r.FormValue("name")
  var key *datastore.Key
  var world *World
  var data []byte
  data, err = zip([]byte(r.FormValue("data")));
  if err != nil {
    c.Errorf("Could not compress data with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  key, world, err = getWorld(c, cur_user, name)
  if err == nil {
    world.Data = data
    _, err = datastore.Put(c, key, world)
  } else {
    world = &World{ cur_user, name, 2, data }
    _, err = datastore.Put(c, datastore.NewIncompleteKey(c, "world", nil),
                           world)
  }
  if err != nil {
    c.Errorf("Could not save world %s for user %s with error: %s",
             name, cur_user, err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
}
