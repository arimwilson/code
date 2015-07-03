package neural

func NewNeuron(function ActivationFunction) *Neuron {
  neuron := new(Neuron)
  neuron.ActivationFunction = function
  return neuron
}

type Neuron struct {
  InputSynapses []*Synapse
  OutputSynapses []*Synapse
  ActivationFunction ActivationFunction
  Output float64
}

func (self *Neuron) ConnectTo(to *Neuron) {
  synapse := NewSynapse(0)
  self.OutputSynapses = append(self.OutputSynapses, synapse)
  to.InputSynapses = append(to.InputSynapses, synapse)
}

func (self *Neuron) Forward() {
  output := 0.0
  for _, synapse := range self.InputSynapses {
    output += synapse.Output
  }
  self.Output = self.ActivationFunction(output)
  for _, synapse := range self.OutputSynapses {
    synapse.Signal(self.Output)
  }
}

