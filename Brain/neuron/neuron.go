package neuron

import ("math"; "math/rand";
        "./synapse")

// ReLU neuron.
type Neuron struct {
  Inputs []float64
  Parameters []float64
  Gradients []float64
  Output float64
}

// parameters is number ffor the neuron.
func New(parameters int) *Neuron {
  neuron := new(Neuron)
  // Initialize parameters randomly from (-0.5, 0.5).
  for i := 0; i < parameters; i++ {
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

