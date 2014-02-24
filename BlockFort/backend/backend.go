package meepesh

import (
  "appengine"; "appengine/datastore"; "appengine/user"; "bytes";
  "compress/gzip"; "encoding/json"; "io/ioutil"; "net/http"
)

func init() {
  http.HandleFunc("/load", load)
  http.HandleFunc("/save", save)
}

type World struct {
  User string
  Name string
  Version int
  Data []byte
}

func getWorld(c appengine.Context, cur_user string, name string) (
    *datastore.Key, *World, error) {
  query := datastore.NewQuery("world").
      Filter("User =", cur_user).
      Filter("Name =", name)
  world := new(World)
  key, err := query.Run(c).Next(world)
  return key, world, err
}

func unzip(compressed_bytes []byte) ([]byte, error) {
  buffer := bytes.NewBuffer(compressed_bytes)
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
  id := r.FormValue("id")
  var world *World
  if name != "" {
    _, world, err = getWorld(c, cur_user, name)
    if err != nil {
       c.Infof("Could not load world %s for user %s: %s.", name, cur_user,
               err.Error())
      if err == datastore.Done {
        http.Error(w, err.Error(), http.StatusBadRequest)
      } else {
        http.Error(w, err.Error(), http.StatusInternalServerError)
      }
      return
    }
  } else if id != "" {
    var key *datastore.Key
    key, err = datastore.DecodeKey(id)
    if err != nil {
      c.Infof("Could not decode key for world with id %s: %s.", id, err.Error())
      http.Error(w, err.Error(), http.StatusBadRequest)
      return
    }
    err = datastore.Get(c, key, world)
    if err != nil {
      c.Infof("Could not load world with id %s: %s.", id, err.Error())
      http.Error(w, err.Error(), http.StatusBadRequest)
      return
    }
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

func zip(uncompressed_bytes []byte) ([]byte, error) {
  buffer := new(bytes.Buffer)
  writer := gzip.NewWriter(buffer)
  _, err := writer.Write(uncompressed_bytes)
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
  var data []byte
  data, err = zip([]byte(r.FormValue("data")));
  if err != nil {
    c.Errorf("Could not compress data with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  var key *datastore.Key
  var world *World
  key, world, err = getWorld(c, cur_user, name)
  if err == nil {
    world.Data = data
    _, err = datastore.Put(c, key, world)
  } else {
    world.User = cur_user
    world.Name = name
    world.Version = 2
    world.Data = data
    key = datastore.NewIncompleteKey(c, "world", nil)
    _, err = datastore.Put(c, key, world)
  }
  if err != nil {
    c.Errorf("Could not save world %s for user %s with error: %s",
             name, cur_user, err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  w.Write([]byte(key.Encode()))
}
