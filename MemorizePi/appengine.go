// +build appengine

package appengine

import (
  "appengine";
  "appengine/urlfetch";
  "fmt";
  "io/ioutil";
  "net/http";
)

func init() {
  http.HandleFunc("/compare", compare_handler)
}

// TODO(ariw): This is very similar to compare() in cmdline.go. Consolidate
// these two functions.
func compare(real_pi []byte, guessed_pi string) []byte {
  i := 0
  for ; i < len(guessed_pi); i++ {
    if real_pi[i] != guessed_pi[i] {
      return []byte(fmt.Sprintf(
        "Wrong on digit %d. You typed %c but it should have been %c.", i - 1,
        guessed_pi[i], real_pi[i]))
    }
  }
  return []byte(fmt.Sprintf(
    "Correct for %d digits of pi. Next 5 digits are %s.", i - 2,
    real_pi[i:i+5]))
}

func compare_handler(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get params from request.
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  client := urlfetch.Client(c)
  resp, err := client.Get("http://memorizepi.appspot.com/pi")
  if err != nil {
    c.Errorf("Could not retrieve pi with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  defer resp.Body.Close()
  var real_pi []byte
  real_pi, err = ioutil.ReadAll(resp.Body)
  if err != nil {
    c.Errorf("Could not read pi with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
  }
  w.Write(compare(real_pi, r.FormValue("guessedPi")))
}

