// Neural network training & execution on a simple supervised regression
// problem.
//
// Sample usage:
// go run brain.go -training_file training.txt -testing_file testing.txt

package main

import ("encoding/json"; "flag"; "fmt"; "io/ioutil"; "log";
        "./neural")

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
  fmt.Printf("output: %v\n", neuralNetwork[1][0].Output)
}

func Train(datapoints []Datapoint) [][]neural.Neuron {
  // Set up an example fully connected network with 2 layers: 2 hidden neural.
  //  1 output neural.
  // TODO(ariw): Modularize input, output, layer, neural network.
  neuralNetwork := make([][]neural.Neuron, 3)
  neuralNetwork[0] = append(neuralNetwork[0], *neural.NewNeuron(2))
  neuralNetwork[0] = append(neuralNetwork[0], *neural.NewNeuron(2))
  neuralNetwork[1] = append(neuralNetwork[1], *neural.NewNeuron(2))
  for i := 0; i < len(datapoints); i++ {
    Forward(neuralNetwork, datapoints[i])
    neuralNetwork[1][0].Backward(
        datapoints[i].Value - neuralNetwork[1][0].Output)
    neuralNetwork[0][0].Backward(neuralNetwork[1][0].Gradients[0])
    neuralNetwork[0][1].Backward(neuralNetwork[1][0].Gradients[1])
  }
  return neuralNetwork
}

func Evaluate(neuralNetwork [][]neural.Neuron, datapoints []Datapoint) {
  for i := 0; i < len(datapoints); i++ {
   Forward(neuralNetwork, datapoints[i])
   fmt.Printf("Training example %v: actual value %v, model value %v\n",
              i, datapoints[i].Value, neuralNetwork[1][0].Output)
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
  // Train the model.
  trainingExamples := ReadDatapointsOrDie(*trainingExamplesFlag)
  neuralNetwork := Train(trainingExamples)

  // Test the model.
  testingExamples := ReadDatapointsOrDie(*testingExamplesFlag)
  Evaluate(neuralNetwork, testingExamples)
}
