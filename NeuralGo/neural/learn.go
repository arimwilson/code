package neural

import ("math/rand")

type Datapoint struct {
  Features []float64
  Values[] float64
}

func Train(neuralNetwork *Network, datapoints []Datapoint, iter int,
           trainingSpeed float64) {
  // Train on some number of iterations of permuted versions of the input.
  for i := 0; i < iter; i++ {
    perm := rand.Perm(len(datapoints))
    for _, index := range perm {
      neuralNetwork.Train(
          datapoints[index].Features, datapoints[index].Values, trainingSpeed)
    }
  }
}

func Evaluate(neuralNetwork *Network, datapoints []Datapoint) float64 {
  square_error := 0.0
  for _, datapoint := range datapoints {
    output := neuralNetwork.Evaluate(datapoint.Features)
    for i, value := range datapoint.Values {
      square_error += (value - output[i]) * (value - output[i])
    }
  }
  return square_error / float64(len(datapoints))
}
