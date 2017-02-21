from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import argparse
import datetime
import collections
import csv
import sys

import numpy as np
import tensorflow as tf

FLAGS = None

Dataset = collections.namedtuple('Dataset', ['data', 'target'])

def load_csv_with_header(filename,
                         target_dtype,
                         features_dtype,
                         target_column=-1,
                         feature_columns=None,
                         feature_preprocessors=[]):
  """Load dataset from CSV file with a header row."""
  with tf.platform.gfile.Open(filename) as csv_file:
    data_file = csv.reader(csv_file)
    header = next(data_file)
    target = []
    data = []
    data_file.next()
    for row in data_file:
        features = []
        for j, feature in enumerate(row):
            k = len(features)
            if j in feature_columns:
                if feature_preprocessors[k]:
                    features.append(feature_preprocessors[k](feature))
                else:
                    features.append(feature)
        data.append(tuple(features))
        target.append(row.pop(target_column))
  return Dataset(data=np.asarray(data, dtype=features_dtype),
                 target=np.asarray(target, dtype=target_dtype))

def float32_or_none(float_str):
    try:
        return np.float32(float_str)
    except ValueError:
        return None

# Return number of seconds from date_str to the present.
def seconds_in_past(date_str):
    return (datetime.datetime.now() -
            datetime.datetime.strptime(date_str, "%Y-%m-%d")).total_seconds()

def feature_normalize(dataset):
    mu = np.mean(dataset, axis=0)
    sigma = np.std(dataset, axis=0)
    return (dataset - mu) / sigma

def read(fuelly_csv_file):
    # Format is car name, model, mpg, miles, gallons, price, city percentage
    # fuelup date, date added, tags, notes, missed fuelup, partial fuelup
    # latitude, longitude, and brand.
    #
    # We use miles, price, city percentage, and fuelup date as features and
    # mpg as target.
    dataset = load_csv_with_header(
        fuelly_csv_file, np.float32, np.float32, 2, [3, 5, 6, 7],
        [None, None, float32_or_none, seconds_in_past])
    # Fill in missing city percentages with sample mean (MCAR approach).
    averages = np.nanmean(dataset.data, axis=0)
    indices = np.where(np.isnan(dataset.data))
    dataset.data[indices] = np.take(averages, indices[1])
    # Normalize all features to mean 0 & distance from standard deviation.
    sigma = np.std(dataset.data, axis=0)
    dataset.data[...] = (dataset.data - averages) / sigma
    return dataset

def evaluate(sess, model, dataset):
    pass

def main(_):
    # Read & parse file into appropriate features & value.
    dataset = read(FLAGS.fuelly_csv_file)

    # Train & eval model
    # Linear model with weight term.
    dim = dataset.data.shape[1]
    X = tf.placeholder(tf.float32, [None, dim])
    Y = tf.placeholder(tf.float32, [None, 1])
    W = tf.Variable(np.random.randn(dim, 1), dtype=tf.float32)
    model = tf.matmul(X, W)
    with tf.Session() as sess:
        tf.global_variables_initializer().run()
        training_step = tf.train.GradientDescentOptimizer(
            FLAGS.learning_rate).minimize(
                tf.reduce_mean(tf.square(model - Y)))
        for i in range(FLAGS.num_epochs):
            sess.run(training_step, feed_dict={X: dataset.data, Y: dataset.target})
        evaluate(sess, model, dataset)
        # TODO(ariw): Add capability to test trained model on new examples.

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--fuelly_csv_file', type=str, default='/tmp/fuelups.csv',
        help='Location of exported Fuelly CSV file')
    parser.add_argument(
        '--learning_rate', type=float, default=0.01,
        help='Learning rate for gradient descent optimization.')
    parser.add_argument(
        '--num_epochs', type=int, default=10,
        help='Number of training epochs.')
    FLAGS, unparsed = parser.parse_known_args()
    tf.app.run(main=main, argv=[sys.argv[0]] + unparsed)
