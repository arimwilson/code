package neural

func NewLayer(neurons int, function ActivationFunction) *Layer {
  layer := new(Layer)
  for i := 0; i < neurons; i++ {
    layer.Neurons = append(layer.Neurons, NewNeuron(function))
  }
  return layer
}

type Layer struct {
  Neurons []*Neuron
}

func (self* Layer) ConnectTo(layer *Layer) {
  for _, neuronFrom := range self.Neurons {
    // Fully connected between layers.
    for _, neuronTo := range layer.Neurons {
      neuronFrom.ConnectTo(neuronTo)
    }
  }
}

func (self* Layer) Forward() {
  for _, neuron := range self.Neurons {
    neuron.Forward()
  }
}
