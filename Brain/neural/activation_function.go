package neural

import ("math")

type ActivationFunction func(float64) float64

func ReLUFunction(x float64) float64 {
  return math.Max(0, x)
}

