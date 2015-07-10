package neural

import ("encoding/json"; "math/rand")

func NewNetwork(
    inputs int, layers []int, functions []string) *Network{
  network := new(Network)
  network.init(inputs, layers, functions)
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

func (self *Network) Evaluate(inputs []float64) []float64 {
  for i, input := range inputs {
    self.Inputs[i].Input = input
    self.Inputs[i].Forward()
  }
  for _, layer := range self.Layers {
    layer.Forward()
  }
  outputLayer := self.Layers[len(self.Layers) - 1]
  outputs := make([]float64, len(outputLayer.Neurons))
  for i, neuron := range(outputLayer.Neurons) {
    outputs[i] = neuron.Output
  }
  return outputs
}

func (self *Network) Train(inputs []float64, values []float64, speed float64) {
  self.Evaluate(inputs)
  outputLayer := self.Layers[len(self.Layers) - 1]
  for i, neuron := range outputLayer.Neurons {
    neuron.BackwardOutput(values[i])
  }
  for i := len(self.Layers) - 2; i >= 0; i-- {
    self.Layers[i].Backward()
  }
  for _, layer := range self.Layers {
    layer.Update(speed)
  }
}

type SerializedNetwork struct {
  Inputs int
  ActivationFunctions []string
  Weights [][][]float64
}

func (self *Network) Serialize() []byte {
  serialized_network := &SerializedNetwork{
      Inputs: len(self.Inputs),
      Weights: make([][][]float64, len(self.Layers))}
  for i, layer := range self.Layers {
    // All neurons in the same layer have the same activation function.
    serialized_network.ActivationFunctions = append(
        serialized_network.ActivationFunctions,
        layer.Neurons[0].ActivationFunction)
    serialized_network.Weights[i] = make([][]float64, len(layer.Neurons))
    for j, neuron := range layer.Neurons {
      serialized_network.Weights[i][j] = make(
          []float64, len(neuron.InputSynapses))
      for k, synapse := range neuron.InputSynapses {
        serialized_network.Weights[i][j][k] = synapse.Weight
      }
    }
  }
  byte_network, _ := json.Marshal(serialized_network)
  return byte_network
}

func (self *Network) Deserialize(byte_network []byte) {
  serialized_network := &SerializedNetwork{}
  json.Unmarshal(byte_network, serialized_network)
  layers := make([]int, len(serialized_network.Weights))
  for i, layer := range serialized_network.Weights {
    layers[i] =len(layer)
  }
  self.init(serialized_network.Inputs, layers,
            serialized_network.ActivationFunctions)
  // Now initialize all the weights.
  for i, layer := range self.Layers {
    for j, neuron := range layer.Neurons {
      for k, synapse := range neuron.InputSynapses {
        synapse.Weight = serialized_network.Weights[i][j][k]
      }
    }
  }
}

func (self *Network) init(inputs int, layers []int, functions []string) {
  self.Inputs = []*Input{}
  for i := 0; i < inputs; i++ {
    self.Inputs = append(self.Inputs, NewInput())
  }
  self.Layers = []*Layer{}
  for i, count := range layers {
    layer := NewLayer(count, functions[i])
    self.Layers = append(self.Layers, layer)
  }
  // Connect all the layers.
  for _, input := range self.Inputs {
    input.ConnectTo(self.Layers[0])
  }
  for i := 0; i < len(self.Layers) - 1; i++ {
    self.Layers[i].ConnectTo(self.Layers[i+1])
  }
}
