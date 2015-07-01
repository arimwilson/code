package neural_test

import ("testing"; "../neural")

func NumericalGradients(neuron neural.Neuron, inputs []float64) float64 {
  h := 0.001
  neuron.Forward(inputs + h)
  fxh = neuron.Output
  neuron.Forward(inputs)
  fx = neuron.Output
  return (fxh - fx) / h
}

// Compare analytic gradient for neuron with numerical gradient.
func TestNeuron(t *testing.T) {
  neuron := neural.NewNeuron(1, neural.ReLUFunction)
  inputs := []float64{1.5}
  neuron.Forward(inputs)
  neuron.Backward(neuron.Output)
  analytic_gradients := neuron.Gradients
  numerical_gradients := NumericalGradients(neuron, inputs)
  fmt.Printf("analytic gradients: %v\nnumerical gradients: %v\n",
             analytic_gradients, numerical_gradients)
}
