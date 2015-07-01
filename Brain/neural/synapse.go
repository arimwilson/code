package neural

type Synapse struct {
  Weight float64
  In float64
  Out float64
}

func NewSynapse(weight float64) *Synapse {
  return &Synapse{Weight: weight}
}

func NewSynapseFromTo(from, to *Neuron, weight float64) *Synapse {
  synapse := NewSynapse(weight)

//  from.OutSynapses = append(from.OutSynapses, synapse)
//  from.InSynapses = append(to.InSynapses, synapse)
  return synapse
}

func (synapse *Synapse) Signal(value float64) {
  synapse.In = value
  synapse.Out = synapse.Weight * synapse.In
}
