// Neural network training & execution on a simple supervised regression
// problem.

package main

import ("encoding/json"; "flag"; "fmt"; "io/ioutil"; "log"; "math"; "math/rand")

var trainingExamplesFlag = flag.String(
  "training_file", "",
  "File with JSON-formatted array of training examples with values.")
var testingExamplesFlag = flag.String(
  "testing_file", "",
  "File with JSON-formatted array of testing examples with values.")

// ReLU neuron.
type Neuron struct {
  Inputs []float64
  Parameters []float64
  Gradients []float64
  Output float64
}

// n is number of parameters for the neuron.
func NewNeuron(n int) *Neuron {
  neuron := new(Neuron)
  // Initialize parameters randomly from (-0.5, 0.5).
  for i := 0; i < n; i++ {
    neuron.Parameters = append(neuron.Parameters, rand.Float64() - 0.5)
    neuron.Gradients = append(neuron.Gradients, 0.0)
  }
  neuron.Output = 0.0
  return neuron
}

func (self *Neuron) Forward(inputs []float64) {
  i := 0
  self.Inputs = inputs
  for ; i < len(self.Parameters) - 1; i++ {
    self.Output += self.Parameters[i] * self.Inputs[i]
  }
  self.Output += self.Parameters[i]
  self.Output = math.Max(0, self.Output)
}

func (self *Neuron) Backward(pull float64) {
  if self.Output == 0 {
   return
  }
  i := 0
  for ; i < len(self.Parameters) - 1; i++ {
    self.Gradients[i] = self.Inputs[i] * pull - self.Parameters[i]
  }
  self.Gradients[i] = pull - self.Parameters[i]
}

func (self *Neuron) Update() {
  stepSize := 0.01
  for i := 0; i < len(self.Parameters); i++ {
    self.Parameters[i] += stepSize * self.Gradients[i]
  }
}

type Datapoint struct {
  Features []float64
  Value float64
}

func Forward(neuralNetwork [][]Neuron, datapoint Datapoint) {
  neuralNetwork[0][0].Forward(datapoint.Features)
  neuralNetwork[0][1].Forward(datapoint.Features)
  neuralNetwork[1][0].Forward([]float64{
      neuralNetwork[0][0].Output, neuralNetwork[0][1].Output})
}

func Train(datapoints []Datapoint) [][]Neuron {
  // Set up an example fully connected network with 2 layers: 2 hidden neurons
  //  1 output neuron.
  // TODO(ariw): Modularize input, output, layer, neural network.
  neuralNetwork := make([][]Neuron, 3)
  neuralNetwork[0] = append(neuralNetwork[0], *NewNeuron(2))
  neuralNetwork[0] = append(neuralNetwork[0], *NewNeuron(2))
  neuralNetwork[1] = append(neuralNetwork[1], *NewNeuron(2))
  for i := 0; i < len(datapoints); i++ {
    Forward(neuralNetwork, datapoints[i])
    neuralNetwork[1][0].Backward(
        datapoints[i].Value - neuralNetwork[1][0].Output)
    neuralNetwork[0][0].Backward(neuralNetwork[1][0].Gradients[0])
    neuralNetwork[0][1].Backward(neuralNetwork[1][0].Gradients[1])
  }
  return neuralNetwork
}

func Evaluate(neuralNetwork [][]Neuron, datapoints []Datapoint) {
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
