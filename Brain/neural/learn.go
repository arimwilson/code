package neural

import ("fmt"; "math/rand")

type Datapoint struct {
  Features []float64
  Value float64
}

func Train(neuralNetwork *Network, datapoints []Datapoint, iter int,
           trainingSpeed float64) {
  // Train on some number of iterations of permuted versions of the input.
  for i := 0; i < iter; i++ {
    perm := rand.Perm(len(datapoints))
    for _, index := range perm {
      neuralNetwork.Train(
          datapoints[index].Features, []float64{datapoints[index].Value},
          trainingSpeed)
    }
    if (i + 1)  % (iter / 4) == 0 {
      fmt.Printf("Training error on iteration %v: %v\n", i + 1,
                 Evaluate(neuralNetwork, datapoints))
    }
  }
}

func Evaluate(neuralNetwork *Network, datapoints []Datapoint) float64 {
  square_error := 0.0
  for _, datapoint := range datapoints {
    output := neuralNetwork.Evaluate(datapoint.Features)
    square_error += (datapoint.Value - output[0]) *
                    (datapoint.Value - output[0])
  }
  return square_error / float64(len(datapoints))
}
