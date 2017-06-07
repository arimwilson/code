from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import argparse
import csv
import datetime
import math
import sys

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import tensorflow as tf

FLAGS = None

class Dataset:
    def __init__(self, data, target):
        self.data = data
        self.target = target

    # Calculate and store averages and standard deviations of data and target
    # for standardization.
    def standardize_stats(self):
        self.data_averages = np.nanmean(self.data, axis=0)
        self.data_std = np.nanstd(self.data, axis=0)
        self.target_averages = np.nanmean(self.target, axis=0)
        self.target_std = np.nanstd(self.target, axis=0)

def load_csv_with_header(filename,
                         target_dtype,
                         features_dtype,
                         target_column=-1,
                         feature_columns=None,
                         feature_preprocessors=[]):
  """Load dataset from CSV file with a header row."""
  with tf.platform.gfile.Open(filename) as csv_file:
    data_file = csv.reader(csv_file)
    target = []
    data = []
    next(data_file)
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

# Sum up partial fuelups into following fuelup, averaging miles, price,
# city_percentage, and date. Assumes fuelups are ordered by date, descending,
# and that fuelups already have summed mpg values (mpgs are not recalculated).
# If there is no following fuelup (last fuelup is partial), partial fuelups will
# be dropped.
def aggregate_partial_fuelups(dataset):
    aggregated_rows = 0
    mile_sum = 0
    price_sum = 0
    has_city_percentage_sum = True
    city_percentage_sum = 0
    date_sum = 0
    partial_rows = []
    reversed_data = dataset.data[::-1]
    for i, row in enumerate(reversed_data):
        if row[4]:  # Did we have a partial fuelup?
            aggregated_rows += 1
            mile_sum += row[0]
            price_sum += row[1]
            if math.isnan(row[2]):
                has_city_percentage_sum = False
                city_percentage_sum += row[2]
            date_sum += row[3]
            partial_rows.append(len(reversed_data) - i - 1)
        else:
            if aggregated_rows > 0:
                row[0] = (row[0] + mile_sum) / aggregated_rows
                row[1] = (row[1] + price_sum) / aggregated_rows
                if has_city_percentage_sum and not math.isnan(row[2]):
                    row[2] = (row[2] + city_percentage_sum) / aggregated_rows
                else:
                    row[2] = float('NaN')
                row[3] = (row[3] + date_sum) / aggregated_rows
            aggregated_rows = 0
            mile_sum = 0
            price_sum = 0
            has_city_percentage_sum = True
            city_percentage_sum = 0
            date_sum = 0
    dataset.data = np.delete(dataset.data, partial_rows, axis=0)
    dataset.target = np.delete(dataset.target, partial_rows, axis=0)

# Read a Fuelly CSV file into a TensorFlow-usable Dataset.
def read(fuelly_csv_file):
    # Format is car name, model, mpg, miles, gallons, price, city percentage
    # fuelup date, date added, tags, notes, missed fuelup, partial fuelup
    # latitude, longitude, and brand.
    #
    # We use miles, price, city percentage, and fuelup date as features and
    # mpg as target.
    dataset = load_csv_with_header(
        fuelly_csv_file, np.float32, np.float32, 2, [3, 5, 6, 7, 12],
        [None, None, float32_or_none, seconds_in_past, None])
    # Sum up partial fuelups.
    aggregate_partial_fuelups(dataset)
    # Delete partial fuelup field.
    dataset.data = np.delete(dataset.data, [4,], axis=1)
    # TODO(ariw): Append bias terms to data a la
    # https://aqibsaeed.github.io/2016-07-07-TensorflowLR/.
    # Fill in missing city percentages with sample mean (MCAR approach).
    dataset.standardize_stats()
    indices = np.where(np.isnan(dataset.data))
    dataset.data[indices] = np.take(dataset.data_averages, indices[1])
    # Normalize all features to mean 0 & distance from standard deviation.
    dataset.data[...] = \
            (dataset.data - dataset.data_averages) / dataset.data_std
    # Normalize all targets to mean 0 & distance from standard deviation.
    dataset.target[...] = \
            (dataset.target - dataset.target_averages) / dataset.target_std
    return dataset

def evaluate(sess, model, dataset):
    # TODO(ariw): Fill out.
    pass

def main(_):
    # Read & parse file into appropriate features & value.
    dataset = read(FLAGS.fuelly_csv_file)
    if FLAGS.describe:
        dataframe = pd.DataFrame(dataset.data)
        dataframe['target'] = dataset.target
        print(dataframe.describe())
        dataframe = dataframe.sort_values(0, axis=0)
        dataframe.plot(x=0, y='target')
        plt.show()

    # Train & eval model
    # Linear model with weight term.
    dim = dataset.data.shape[1]
    X = tf.placeholder(tf.float32, [None, dim])
    Y = tf.placeholder(tf.float32)
    W = tf.Variable(tf.truncated_normal(
        [dim, 1], mean=0.0, stddev=1.0, dtype=tf.float32))
    model = tf.matmul(X, W)
    cost = tf.reduce_mean(tf.square(Y - model))
    training_step = tf.train.GradientDescentOptimizer(
        FLAGS.learning_rate).minimize(cost)
    with tf.Session() as sess:
        sess.run(tf.global_variables_initializer())
        for i in range(FLAGS.num_epochs):
            feed_dict = {X: dataset.data, Y: dataset.target}
            sess.run(training_step, feed_dict=feed_dict)
            if i % (FLAGS.num_epochs / 5) == 0:
                print("Weights: ", sess.run(W),
                      "\nCost: ", sess.run(cost, feed_dict=feed_dict))
        evaluate(sess, model, dataset)
        # TODO(ariw): Make this easier to use to test new examples.
        test_fuelup = [300, 2.5, 100, seconds_in_past('2013-07-06')]
        test_fuelup = (test_fuelup - dataset.data_averages) / dataset.data_std
        test_mpg = sess.run(model, feed_dict={X: [test_fuelup]})
        print(test_mpg * dataset.target_std + dataset.target_averages)

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
    parser.add_argument(
        '--describe', type=bool, default=False,
        help='Whether to describe data before/during training.')
    FLAGS, unparsed = parser.parse_known_args()
    tf.app.run(main=main, argv=[sys.argv[0]] + unparsed)
