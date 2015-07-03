package neural

import ("math/rand")

func NewNetwork(
    inputs int, layers []int, function ActivationFunction) *Network{
  network := new(Network)
  for i := 0; i < inputs; i++ {
    network.Inputs = append(network.Inputs, NewInput())
  }
  for _, count := range layers {
    layer := NewLayer(count, function)
    network.Layers = append(network.Layers, layer)
  }
  // Connect all the layers.
  for _, input := range network.Inputs {
    input.ConnectTo(network.Layers[0])
  }
  for i := 0; i < len(network.Layers) - 1; i++ {
    network.Layers[i].ConnectTo(network.Layers[i+1])
  }
  return network
}

type Network struct {
  Inputs []*Input
  Layers []*Layer
}

func (self *Network) RandomizeSynapses() {
  for _, layer := range self.Layers {
    for _, neuron := range layer.Neurons {
      for _, synapse := range neuron.InputSynapses {
        synapse.Weight = rand.Float64() - 0.5
      }
    }
  }
}

func (self *Network) Forward() {
  for _, layer := range self.Layers {
    layer.Forward()
  }
}

func (self *Network) Calculate(inputs []float64) []float64 {
  for i, input := range inputs {
    self.Inputs[i].Input = input
    self.Inputs[i].Forward()
  }
  self.Forward()
  outputLayer := self.Layers[len(self.Layers) - 1]
  outputs := make([]float64, len(outputLayer.Neurons))
  for i, neuron := range(outputLayer.Neurons) {
    outputs[i] = neuron.Output
  }
  return outputs
}

