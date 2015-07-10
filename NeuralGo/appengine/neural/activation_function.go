package neural

import ("math")

func ActivationFunction(name string, x float64) float64 {
  if (name == "Linear") {
    return x
  } else if (name == "ReLU") {
    return math.Max(0, x)
  } else if (name == "Logistic") {
    return 1 / (1 + math.Exp(-x))
  } else if (name == "Tanh") {
    return math.Tanh(x)
  } else {
    return 0
  }
}

func DActivationFunction(name string, y float64) float64 {
  if (name == "Linear") {
    return 1
  } else if (name == "ReLU") {
    if y <= 0 {
      return 0
    }
    return 1
  } else if (name == "Logistic") {
    logistic := ActivationFunction(name, y)
    return logistic * (1 - logistic)
  } else if (name == "Tanh") {
    return 1 - ActivationFunction(name, y) * ActivationFunction(name, y)
  } else {
    return 0
  }
}

