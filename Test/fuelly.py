from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import argparse
import csv
import sys

import tensorflow as tf

FLAGS = None

def read():
  pass

def model():
  pass

def train():
  pass

def evaluate():
  pass

def main(_):
  # Read & parse file into appropriate features.
  data = read(FLAGS.fuelly_csv_file)

  # Train & eval simple neural network regression model.
  trained_model = train(model(), data)
  evaluate(trained_model, data)

if __name__ == '__main__':
  parser = argparse.ArgumentParser()
  parser.add_argument(
      '--fuelly_csv_file', type=str, default='/tmp/fuelups.csv',
      help='Location of exported Fuelly CSV file')
  FLAGS, unparsed = parser.parse_known_args()
  tf.app.run(main=main, argv=[sys.argv[0]] + unparsed)
