package neural

import ("math/rand")

// ReLU neuron.
type Neuron struct {
  ActivationFunction ActivationFunction
  Inputs []float64
  Parameters []float64
  Gradients []float64
  Output float64
}

// parameters is number for the neuron.
func NewNeuron(parameters int, function ActivationFunction) *Neuron {
  neuron := new(Neuron)
  neuron.ActivationFunction = function
  // Initialize parameters randomly from (-0.5, 0.5).
  for i := 0; i < parameters; i++ {
    neuron.Parameters = append(neuron.Parameters, rand.Float64() - 0.5)
    neuron.Gradients = append(neuron.Gradients, 0.0)
  }
  neuron.Output = 0
  return neuron
}

func (self *Neuron) Forward(inputs []float64) {
  i := 0
  self.Inputs = inputs
  output := 0.0
  for ; i < len(self.Parameters) - 1; i++ {
    output += self.Parameters[i] * self.Inputs[i]
  }
  output += self.Parameters[i]
  self.Output = self.ActivationFunction(output)
}

func (self *Neuron) Backward(pull float64) {
  if self.Output == 0 {
   return
  }
  i := 0
  for ; i < len(self.Inputs) - 1; i++ {
    self.Gradients[i] = self.Inputs[i] * pull
  }
  self.Gradients[i] = pull
}

func (self *Neuron) Update(stepSize float64) {
  for i := 0; i < len(self.Parameters); i++ {
    self.Parameters[i] += stepSize * (self.Gradients[i] - self.Parameters[i])
  }
}

