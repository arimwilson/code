package neural

func NewInput() *Input {
  return &Input{}
}

type Input struct {
  OutputSynapses []*Synapse
  Input float64
}

func (input *Input) ConnectTo(layer *Layer) {
  // All inputs connect to all neurons in this layer.
  for _, neuron := range layer.Neurons {
    synapse := NewSynapse(0)
    input.OutputSynapses = append(input.OutputSynapses, synapse)
    neuron.InputSynapses = append(neuron.InputSynapses, synapse)
  }
}

func (input *Input) Forward() {
  for _, synapse := range input.OutputSynapses {
    synapse.Signal(input.Input)
  }
}
