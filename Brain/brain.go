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
var testingExamplesFlag = flag.String(
  "testing_file", "",
  "File with JSON-formatted array of testing examples with values.")

type Datapoint struct {
  Features []float64
  Value float64
}

func Forward(neuralNetwork [][]neural.Neuron, datapoint Datapoint) {
  neuralNetwork[0][0].Forward(datapoint.Features)
  neuralNetwork[0][1].Forward(datapoint.Features)
  neuralNetwork[1][0].Forward([]float64{
      neuralNetwork[0][0].Output, neuralNetwork[0][1].Output})
}

func Train(datapoints []Datapoint) [][]neural.Neuron {
  // Set up an example fully connected network with 2 layers: 2 hidden neural.
  //  1 output neural.
  // TODO(ariw): Modularize input, output, layer, neural network.
  neuralNetwork := make([][]neural.Neuron, 3)
  neuralNetwork[0] = append(neuralNetwork[0], *neural.NewNeuron(2, neural.ReLUFunction))
  neuralNetwork[0] = append(neuralNetwork[0], *neural.NewNeuron(2, neural.ReLUFunction))
  neuralNetwork[1] = append(neuralNetwork[1], *neural.NewNeuron(2, neural.ReLUFunction))
  for iter := 0; iter < 1000; iter++ {
    i := rand.Int() % len(datapoints)
    Forward(neuralNetwork, datapoints[i])
    fmt.Printf("Actual value: %v, prev model: %v\n", datapoints[i].Value, neuralNetwork[1][0].Output)
    neuralNetwork[1][0].Backward(
        datapoints[i].Value - neuralNetwork[1][0].Output)
    stepSize := 0.001
    neuralNetwork[1][0].Update(stepSize)
    neuralNetwork[0][0].Backward(neuralNetwork[1][0].Gradients[0])
    neuralNetwork[0][0].Update(stepSize)
    neuralNetwork[0][1].Backward(neuralNetwork[1][0].Gradients[1])
    neuralNetwork[0][1].Update(stepSize)
    Forward(neuralNetwork, datapoints[i])
    fmt.Printf("Current value: %v\n", neuralNetwork[1][0].Output)
  }
  return neuralNetwork
}

func Evaluate(neuralNetwork [][]neural.Neuron, datapoints []Datapoint) {
  for i := 0; i < len(datapoints); i++ {
   Forward(neuralNetwork, datapoints[i])
   fmt.Printf("Testing example %v: actual value %v, model value %v\n",
              datapoints[i].Features, datapoints[i].Value,
              neuralNetwork[1][0].Output)
  }
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

  // Train the model.
  trainingExamples := ReadDatapointsOrDie(*trainingExamplesFlag)
  neuralNetwork := Train(trainingExamples)

  // Test the model.
  testingExamples := ReadDatapointsOrDie(*testingExamplesFlag)
  Evaluate(neuralNetwork, testingExamples)
}
