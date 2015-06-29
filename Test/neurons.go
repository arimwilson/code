// Neural network training & execution on a simple supervised regression
// problem.

package main

import ("encoding/json"; "flag"; "fmt"; "log"; "math"; "math/rand")

var trainingExamplesFlag = flag.String(
  "training", "",
  "JSON-formatted array of training examples with values.")
var testingExamplesFlag = flag.String(
  "testing", "",
  "JSON-formatted array of testing examples with values.")

// ReLU neuron.
type Neuron struct {
  Inputs []float64
  Parameters []float64
  Gradients []float64
  Output float64
}

// n is number of parameters for the neuron.
func NewNeuron(n int) *Neuron {
  n := new(Neuron)
  // Initialize parameters randomly from (-0.5, 0.5).
  for i := 0; i < n; i++ {
    n.Parameters = append(n.Parameters, rand.Float64() - 0.5)
    n.Gradients = append(n.Gradients, 0.0)
  }
  n.Output = 0.0
  return n
}

func (self *Neuron) Forward(inputs []float64) {
  i := 0
  self.Inputs = inputs
  for ; i < len(self.Parameters) - 1; i++ {
    self.Output += self.Parameters[i] * self.Inputs[i]
  }
  self.Output += self.Parameters[i]
}

func (self *Neuron) Backward(pull float64) {
  if self.Output == 0 {
   return
  }
  i := 0
  for ; i < len(self.Inputs); i++ {
    self.Gradients[i] = self.Inputs[i] * pull - self.Parameters[i]
  }
  self.Gradients[i] = pull - self.Parameters[i]
}

func (self *Neuron) Update() {
  stepSize := 0.01
  for i := 0; i < len(Parameters); i++ {
    self.Parameters += stepSize * self.Gradients[i]
  }
}

type Datapoint struct {
  Features []float64
  Label bool
}

func Train(datapoints []Datapoint) {
  // Set up an example fully connected network with 2 layers: 2 hidden neurons
  //  1 output neuron.
  neuralNetwork := make([][]Neuron, 2)
  neuralNetwork[0] = append(neuralNetwork[0], NewNeuron(2))
  neuralNetwork[0] = append(neuralNetwork[0], NewNeuron(2))
  neuralNetwork[1] = append(neuralNetwork[1], NewNeuron(2))
  for i := 0; i < len(datapoints); i++ {
    neuralNetwork[0][0].Forward(datapoints[i].Features)
    neuralNetwork[0][1].Forward(datapoints[i].Features)
    neuralNetwork[1][0].Forward({
        neuralNetwork[0][0].Output, neuralNetwork[0][1].Output})
    neuralNetwork[1][0].Backward(
        datapoints[i].Output - neuralNetwork[1][0].Output)
    neuralNetwork[0][0].Backward(neuralNetwork[1][1].Gradients[0])
    neuralNetwork[0][1].Backward(neuralNetwork[1][1].Gradients[1])
  }
}

func Evaluate(datapoints []Datapoint) {
  for i := 0; i < len(datapoints); i++ {
  }
}

func main() {
  flag.Parse()
  // Train the model.
  trainingExamples := make([]Datapoint, 0)
  err := json.Unmarshal([]byte(trainingExamplesFlag), &trainingExamples)
  if err != nil {
    log.Fatal(err)
  }
  model = Train(testingExamples)

  // Test the model.
  testingExamples := make([]Datapoint, 0)
  err := json.Unmarshal([]byte(testingExamplesFlag), &testingExamples)
  if err != nil {
    log.Fatal(err)
  }
  Evaluate(model, testingExamples)
}
