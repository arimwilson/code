package neural

func NewNeuron(function string) *Neuron {
  neuron := new(Neuron)
  // Add bias term as first input.
  synapse := NewSynapse(0)
  neuron.InputSynapses = append(neuron.InputSynapses, synapse)
  neuron.ActivationFunction = function
  return neuron
}

type Neuron struct {
  InputSynapses []*Synapse
  OutputSynapses []*Synapse
  ActivationFunction string
  Output float64
}

func (self *Neuron) ConnectTo(to *Neuron) {
  synapse := NewSynapse(0)
  self.OutputSynapses = append(self.OutputSynapses, synapse)
  to.InputSynapses = append(to.InputSynapses, synapse)
}

func (self *Neuron) Forward() {
  output := 0.0
  for i, synapse := range self.InputSynapses {
    // Bias term is not activated by previous layer; we need to manually signal.
    if i == 0 {
      synapse.Signal(1)
    }
    output += synapse.Output
  }
  self.Output = ActivationFunction(self.ActivationFunction, output)
  for _, synapse := range self.OutputSynapses {
    synapse.Signal(self.Output)
  }
}

func (self* Neuron) Backward() {
  gradient := 0.0
  for _, synapse := range self.OutputSynapses {
    gradient += synapse.Weight * synapse.Gradient
  }
  gradient =
      DActivationFunction(self.ActivationFunction, self.Output) * gradient
  for _, synapse := range self.InputSynapses {
    synapse.Feedback(gradient)
  }
}

func (self* Neuron) BackwardOutput(value float64) {
  gradient := DActivationFunction(self.ActivationFunction, self.Output) *
              (value - self.Output)
  for _, synapse := range self.InputSynapses {
    synapse.Feedback(gradient)
  }
}

func (self* Neuron) Update(speed float64) {
  for _, synapse := range self.InputSynapses {
    synapse.Update(speed)
  }
}
