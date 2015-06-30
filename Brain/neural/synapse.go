package neural

func NewSynapse(weight float64) *Synapse {
  return &Synapse{Weight: weight}
}

func NewSynapseFromTo(from, to *Neuron, weight float64) *Synapse {
  synapse := NewSynapse(weight)

  
}
