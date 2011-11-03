// Find new interesting items from an RSS feed.

package interesaint

import ("appengine"; "appengine/datastore"; "appengine/taskqueue";
        "appengine/urlfetch"; "appengine/user"; "http"; "time")

func init() {
  http.HandleFunc("/add", add)
  http.HandleFunc("/tasks/update", update)
}

type Feed struct {
  Url string
}

type User struct {
  Username string
}

type Subscription struct {
  UserId string
  FeedId string
}

type Item struct {
  FeedId string
  Date string
  Content string
}

type Rating struct {
  UserId string
  ItemId string
  interesting float64
}

func add(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }

  cur_user := new(User)
  cur_user.Username = user.Current(c).String()
  query := datastore.NewQuery("user").
      Filter("Username =", cur_user.Username)
  var user_id *datastore.Key
  user_id, err = query.Run(c).Next(cur_user)
  // Do we have a user already?
  if err == datastore.Done {
    user_id = datastore.NewIncompleteKey(c, "user", nil)
    _, err = datastore.Put(c, user_id, cur_user)
    if err != nil {
      c.Errorf("Unable to store user %s with error: %s",
               cur_user.Username, err.String())
      http.Error(w, err.String(), http.StatusInternalServerError)
    }
  } else if err != nil {
    c.Errorf("Unable to look up user %s with error: %s", cur_user.Username,
             err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }

  feed := new(Feed)
  feed.Url = r.FormValue("url")
  query = datastore.NewQuery("feed").
      Filter("Url =", feed.Url)
  var feed_id *datastore.Key
  feed_id, err = query.Run(c).Next(feed)
  // Do we have a feed already?
  if err == datastore.Done {
    feed_id = datastore.NewIncompleteKey(c, "feed", nil)
    _, err = datastore.Put(c, feed_id, feed)
    if err != nil {
      c.Errorf("Unable to store feed %s with error: %s", feed.Url, err.String())
      http.Error(w, err.String(), http.StatusInternalServerError)
      return
    }
  } else if err != nil {
    c.Errorf("Unable to look up feed %s with error: %s", feed.Url, err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }

  subscription := new(Subscription)
  subscription.UserId = user_id.Encode()
  subscription.FeedId = feed_id.Encode()
  query = datastore.NewQuery("subscription").
      Filter("UserId =", subscription.UserId).
      Filter("FeedId =", subscription.FeedId)
  _, err = query.Run(c).Next(subscription)
  // Do we have a subscription already?
  if err == datastore.Done {
    _, err = datastore.Put(
        c, datastore.NewIncompleteKey(c, "subscription", nil), subscription)
    if err != nil {
      c.Errorf("Unable to store subscription for user %s to feed %s with " +
               "error: %s", cur_user.Username, feed.Url, err.String())
      http.Error(w, err.String(), http.StatusInternalServerError)
      return
    }
  } else if err != nil {
    c.Errorf("Unable to look up subscription with error: %s", err.String())
    http.Error(w, err.String(), http.StatusInternalServerError)
    return
  }
}

func update(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  t := datastore.NewQuery("feed").Run(c)
  feed := new(Feed)
  client := urlfetch.Client(c)
  for _, err := t.Next(feed); err != datastore.Done; _, err = t.Next(feed) {
    if err != nil {
      c.Errorf("Unable to get subscription with error: %s", err.String())
      return
    }
    var resp *http.Response
    resp, err = client.Get(feed.Url)
    if err != nil {
      c.Errorf("Unable to refresh subscription to %s with error: %s",
               feed.Url, err.String())
      continue
    }
    defer resp.Body.Close()
    // TODO(ariw): Fix!
    example := make([]byte, 50)
    n, _ := resp.Body.Read(example)
    c.Infof(string(example[:n]))
  }

  // Let's go to sleep for a bit to prevent constant refreshing.
  // TODO(ariw): Cronjobs rather than task queue?
  time.Sleep(10e9)
  next := taskqueue.NewPOSTTask("/tasks/update", nil)
  // Keep trying to re-add ourselves until we succeed.
  for _, err := taskqueue.Add(c, next, "update"); err != nil; {
  }
}

