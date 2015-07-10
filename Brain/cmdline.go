// Feed-forward neural network training & execution on a simple supervised
// regression problem.
//
// Sample usage:
// go run cmdline.go -training_file training.txt -testing_file testing.txt

package main

import ("encoding/json"; "flag"; "fmt"; "io/ioutil"; "log"; "math/rand";
        "time"; "./neural")

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

  // Set up an example neural network with 2 inputs and 1 output.
  neuronsNumber := make([]int, 0)
  err := json.Unmarshal([]byte(*neuronsNumberFlag), &neuronsNumber)
  if err != nil {
    log.Fatal(err)
  } else if neuronsNumber[len(neuronsNumber) - 1] != 1 {
    log.Fatal("Unexpected number of output neurons; expected 1!")
  }
  neuronsFunction := make([]string, 0)
  err = json.Unmarshal([]byte(*neuronsFunctionFlag), &neuronsFunction)
  if err != nil {
    log.Fatal(err)
  }
  neuralNetwork := neural.NewNetwork(2, neuronsNumber, neuronsFunction)
  neuralNetwork.RandomizeSynapses()

  // Train the model.
  trainingExamples := ReadDatapointsOrDie(*trainingExamplesFlag)
  neural.Train(neuralNetwork, trainingExamples, *trainingIterationsFlag,
        *trainingSpeedFlag)

  // Test the model.
  testingExamples := ReadDatapointsOrDie(*testingExamplesFlag)
  fmt.Printf("Testing error: %v\nFinal network: %v\n",
             neural.Evaluate(neuralNetwork, testingExamples),
             string(neuralNetwork.Serialize()))
}
