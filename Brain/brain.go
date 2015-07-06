// Neural network training & execution on a simple supervised regression
// problem.
//
// Sample usage:
// go run brain.go -training_file training.txt -testing_file testing.txt

package main

import ("encoding/json"; "flag"; "fmt"; "io/ioutil"; "log"; "math/rand";
        "time"; "./neural")

var trainingExamplesFlag = flag.String(
  "training_file", "",
  "File with JSON-formatted array of training examples with values.")
var hiddenNeuronsNumberFlag = flag.Int(
  "hidden_neurons_number", 10, "Number of hidden neurons in the network.")
var hiddenNeuronsFunctionFlag = flag.String(
  "hidden_neurons_function", "ReLU", "Activation function for hidden neurons.")
var trainingIterationsFlag = flag.Int(
  "training_iterations", 1000, "Number of training iterations.")
var trainingSpeedFlag = flag.Float64(
  "training_speed", 0.001, "Speed of training.")
var testingExamplesFlag = flag.String(
  "testing_file", "",
  "File with JSON-formatted array of testing examples with values.")

type Datapoint struct {
  Features []float64
  Value float64
}

func Train(neuralNetwork *neural.Network, datapoints []Datapoint, iter int) {
  // Train on some number of iterations of permuted versions of the input.
  for i := 0; i < iter; i++ {
    perm := rand.Perm(len(datapoints))
    for _, index := range perm {
      neuralNetwork.Train(
          datapoints[index].Features, []float64{datapoints[index].Value},
          *trainingSpeedFlag)
    }
    if (i + 1)  % (iter / 4) == 0 {
      fmt.Printf("Training error on iteration %v: %v\n", i + 1,
                 Evaluate(neuralNetwork, datapoints))
    }
  }
}

func Evaluate(neuralNetwork *neural.Network, datapoints []Datapoint) float64 {
  square_error := 0.0
  for _, datapoint := range datapoints {
    output := neuralNetwork.Evaluate(datapoint.Features)
    square_error += (datapoint.Value - output[0]) *
                    (datapoint.Value - output[0])
  }
  return square_error / float64(len(datapoints))
}

func ReadDatapointsOrDie(filename string) []Datapoint {
  bytes, err := ioutil.ReadFile(filename)
  if err != nil {
    log.Fatal(err)
  }
  datapoints := make([]Datapoint, 0)
  err = json.Unmarshal(bytes, &datapoints)
  if err != nil {
    log.Fatal(err)
  }
  return datapoints
}

func main() {
  flag.Parse()
  rand.Seed(time.Now().UTC().UnixNano())

  // Set up an example fully connected network with 2 inputs and 2 layers.
  // Layer 1 consists of n hidden ReLU neurons and layer consists of one output
  // linear neuron.
  neuralNetwork := neural.NewNetwork(2, []int{*hiddenNeuronsNumberFlag, 1},
                                     []string{*hiddenNeuronsFunctionFlag,
                                              "Linear"})
  neuralNetwork.RandomizeSynapses()

  // Train the model.
  trainingExamples := ReadDatapointsOrDie(*trainingExamplesFlag)
  Train(neuralNetwork, trainingExamples, *trainingIterationsFlag)

  // Test the model.
  testingExamples := ReadDatapointsOrDie(*testingExamplesFlag)
  fmt.Printf("Testing error: %v\nFinal network: %v\n",
             Evaluate(neuralNetwork, testingExamples),
             string(neuralNetwork.Serialize()))
}
