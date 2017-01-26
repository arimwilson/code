from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import argparse
import csv
import sys

import tensorflow as tf

FLAGS = None

class Datapoint(object):
    def __init__(self, header, value_header, row):
        self.features = {}
        for i in xrange(len(row)):
            if header[i] == value_header:
                self.value = row[i]
            else:
                 self.features[header[i]] = row[i]

    def __str__(self):
        return "Features: %s, value: %s" % (self.features, self.value)

def read_to_datapoints(fuelly_csv_file):
    reader = csv.reader(open(fuelly_csv_file, 'rb'))
    datapoints = []
    header = reader.next()
    for row in reader:
        datapoints.append(Datapoint(header, 'mpg', row))

def read(fuelly_csv_file):
    filename_queue = tf.train.string_input_producer([fuelly_csv_file])
    reader = tf.TextLineReader()
    key, value = reader.read(filename_queue)
    # Format is car name, model, mpg, miles, gallons, price, city percentage
    # fuelup date, date added, tags, notes, missed_fuelup, partial_fuelup
    # latitude, longitude, and brand.
    record_defaults = [
        [''], [''], [0.0], [0.0], [0.0], [0.0], [0], [''], [''], [''], [''],
        [0], [0], [0.0], [0.0], ['']]
    car_name, model, mpg, miles, gallons, price, city_percentage, fuelup_date, \
        date_added, tags, notes, missed_fuelup, partial_fuelup, latitude, \
        longitude, brand = tf.decode_csv(value,
            record_defaults=record_defaults)
    features = tf.pack([miles, city_percentage, fuelup_date, price])
    print(features)

def model():
    pass

def train(model, data):
    pass

def evaluate(model, data):
    pass

def main(_):
    # Read & parse file into appropriate features & value.
    data = read(FLAGS.fuelly_csv_file)

    # Train & eval simple neural network regression model.
    trained_model = train(model(), data)
    evaluate(trained_model, data)

    # TODO(ariw): Add capability to test trained model on new examples.

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--fuelly_csv_file', type=str, default='/tmp/fuelups.csv',
        help='Location of exported Fuelly CSV file')
    FLAGS, unparsed = parser.parse_known_args()
    tf.app.run(main=main, argv=[sys.argv[0]] + unparsed)
