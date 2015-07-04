package neural

type Synapse struct {
  Weight float64
  Input float64
  Output float64
  Gradient float64
}

func NewSynapse(weight float64) *Synapse {
  return &Synapse{Weight: weight}
}

func (self *Synapse) Signal(value float64) {
  self.Input = value
  self.Output = self.Weight * self.Input
}

func (self *Synapse) Feedback(gradient float64) {
  self.Gradient = gradient
}

func (self *Synapse) Update(speed float64) {
  self.Weight += speed * self.Gradient * self.Input
}
