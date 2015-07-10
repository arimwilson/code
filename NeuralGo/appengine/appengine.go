package appengine

import ("appengine"; "appengine/memcache"; "encoding/json"; "fmt"; "math/rand";
        "net/http"; "strconv"; "time"; "neural")

func init() {
  http.HandleFunc("/train", train)
  http.HandleFunc("/test", test)
}

func unmarshal(data []byte, v interface{}, c appengine.Context,
               w http.ResponseWriter) bool {
  err := json.Unmarshal(data, v)
  if err != nil {
    c.Errorf("Could not unmarshal data with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return false
  }
  return true
}

func train(w http.ResponseWriter, r *http.Request) {
  c := appengine.NewContext(r)
  // Get params from request.
  err := r.ParseForm()
  if err != nil {
    c.Errorf("Could not parse form with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  timeNano := time.Now().UTC().UnixNano()
  rand.Seed(timeNano)

  // Set up neural network using form values.
  neuronsNumber := make([]int, 0)
  if !unmarshal([]byte(r.FormValue("neuronsNumber")), &neuronsNumber, c, w) {
    return
  }
  neuronsFunction := make([]string, 0)
  if !unmarshal([]byte(r.FormValue("neuronsFunction")), &neuronsFunction, c,
                w) {
    return
  }
  trainingExamples := make([]neural.Datapoint, 0)
  if !unmarshal([]byte(r.FormValue("trainingExamples")), &trainingExamples, c,
                w) {
    return
  }
  // Use the first training example to decide how many features we have.
  neuralNetwork := neural.NewNetwork(
      len(trainingExamples[0].Features), neuronsNumber, neuronsFunction)
  neuralNetwork.RandomizeSynapses()
  var trainingIterations int
  trainingIterations, err = strconv.Atoi(r.FormValue("trainingIterations"))
  if err != nil {
    c.Errorf("Could not parse trainingIterations with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  var trainingSpeed float64
  trainingSpeed, err = strconv.ParseFloat(r.FormValue("trainingSpeed"), 64)
  if err != nil {
    c.Errorf("Could not parse trainingSpeed with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }

  // Train the model.
  neural.Train(neuralNetwork, trainingExamples, trainingIterations,
               trainingSpeed)
  modelId := strconv.FormatInt(timeNano, 10)
  item := &memcache.Item{
    Key: modelId,
    Value: neuralNetwork.Serialize(),
  }
  if err = memcache.Add(c, item); err != nil {
    c.Errorf("Could not store neural network with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  w.Write([]byte(fmt.Sprintf(
      "{\"modelId\": \"%v\", \"output\": \"Training error: %v\\n\"}",
      modelId, neural.Evaluate(neuralNetwork, trainingExamples))))
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
  testingExamples := make([]neural.Datapoint, 0)

  // Test the model.
  var byteNetwork *memcache.Item
  if byteNetwork, err = memcache.Get(c, r.FormValue("modelId")); err != nil {
    c.Errorf("Could not retrieve neural network with error: %s", err.Error())
    http.Error(w, err.Error(), http.StatusInternalServerError)
    return
  }
  if !unmarshal([]byte(r.FormValue("testingExamples")), &testingExamples, c,
                w) {
    return
  }
  var neuralNetwork neural.Network
  neuralNetwork.Deserialize(byteNetwork.Value)
  w.Write([]byte(fmt.Sprintf(
    "Testing error: %v\nFinal network: %v\n",
    neural.Evaluate(&neuralNetwork, testingExamples),
    string(byteNetwork.Value))))
}

