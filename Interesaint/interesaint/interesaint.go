// Find new interesting items from an RSS feed.

import ("appengine"; "appengine/datastore"; "appengine/urlfetch"; "http")

func init() {
  http.HandleFunc("/add", add)
  http.HandleFunc("/tasks/update", update)
}

type Feed struct {
  Url string
}

type Subscription struct {
  FeedId *datastore.Key
  User string
}

type Item struct {
  FeedId *datastore.Key
  Content string
}

type Rating struct {
  UserId *datastore.Key
  ItemId *datastore.Key
  interesting bool
}

func add(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }

  url := r.FormValue("url")
  query := datastore.NewQuery("feed")
  query.Filter("Url =", url)
  feed := new(Feed)
  query.Run(c).Next(feed)
}

func update(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
}

