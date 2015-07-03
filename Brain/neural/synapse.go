package neural

type Synapse struct {
  Weight float64
  Input float64
  Output float64
}

func NewSynapse(weight float64) *Synapse {
  return &Synapse{Weight: weight}
}

func (synapse *Synapse) Signal(value float64) {
  synapse.Input = value
  synapse.Output = synapse.Weight * synapse.Input
}
