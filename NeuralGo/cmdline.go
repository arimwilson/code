// Feed-forward neural network training & execution on a simple supervised
// regression problem.
//
// Sample usage:
// go run cmdline.go -training_file training.txt -testing_file testing.txt

package main

import ("encoding/json"; "flag"; "fmt"; "io/ioutil"; "log"; "math/rand";
        "time"; "./appengine/neural")

var trainingExamplesFlag = flag.String(
  "training_file", "",
  "File with JSON-formatted array of training examples with values.")
var neuronsNumberFlag = flag.String(
  "neurons_number", "[10, 1]",
  "JSON-formatted array of number of neurons for each layer in the network.")
var neuronsFunctionFlag = flag.String(
  "neurons_function", "[\"ReLU\", \"Linear\"]",
  "JSON-formatted array of activation function for neurons each layer in the " +
  "network.")
var trainingIterationsFlag = flag.Int(
  "training_iterations", 1000, "Number of training iterations.")
var trainingSpeedFlag = flag.Float64(
  "training_speed", 0.001, "Speed of training.")
var testingExamplesFlag = flag.String(
  "testing_file", "",
  "File with JSON-formatted array of testing examples with values.")

func ReadDatapointsOrDie(filename string) []neural.Datapoint {
  bytes, err := ioutil.ReadFile(filename)
  if err != nil {
    log.Fatal(err)
  }
  datapoints := make([]neural.Datapoint, 0)
  err = json.Unmarshal(bytes, &datapoints)
  if err != nil {
    log.Fatal(err)
  }
  return datapoints
}

func main() {
  flag.Parse()
  rand.Seed(time.Now().UTC().UnixNano())

  // Set up neural network using flags.
  neuronsNumber := make([]int, 0)
  err := json.Unmarshal([]byte(*neuronsNumberFlag), &neuronsNumber)
  if err != nil {
    log.Fatal(err)
  }
  neuronsFunction := make([]string, 0)
  err = json.Unmarshal([]byte(*neuronsFunctionFlag), &neuronsFunction)
  if err != nil {
    log.Fatal(err)
  }
  trainingExamples := ReadDatapointsOrDie(*trainingExamplesFlag)
  // Use the first training example to decide how many features we have.
  neuralNetwork := neural.NewNetwork(
      len(trainingExamples[0].Features), neuronsNumber, neuronsFunction)
  neuralNetwork.RandomizeSynapses()
  testingExamples := ReadDatapointsOrDie(*testingExamplesFlag)

  // Train the model.
  neural.Train(neuralNetwork, trainingExamples, *trainingIterationsFlag,
               *trainingSpeedFlag)

  // Test & print model:
  fmt.Printf("Training error: %v\nTesting error: %v\nNetwork: %v\n",
             neural.Evaluate(neuralNetwork, trainingExamples),
             neural.Evaluate(neuralNetwork, testingExamples),
             string(neuralNetwork.Serialize()))
}
