package neural

func NewInput() *Input {
  return &Input{}
}

type Input struct {
  OutputSynapses []*Synapse
  Input float64
}

func (self *Input) ConnectTo(layer *Layer) {
  // All inputs connect to all neurons in the first layer.
  for _, neuron := range layer.Neurons {
    synapse := NewSynapse(0, 0)
    self.OutputSynapses = append(self.OutputSynapses, synapse)
    neuron.InputSynapses = append(neuron.InputSynapses, synapse)
  }
}

func (self *Input) Forward() {
  for _, synapse := range self.OutputSynapses {
    synapse.Signal(self.Input)
  }
}
