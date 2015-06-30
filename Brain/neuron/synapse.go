package neural

func New(weight float64) *Synapse {
  return &Synapse{Weight: weight}
}

func NewFromTo(from, to *Neuron, weight float64) *Synapse {
  synapse := NewSynapse(weight)

  
}
