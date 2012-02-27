import math
import numpy

def break_row(line):
  line = line.split(",", 1)
  return float(line[0]), feature_row(line[1])

def feature_row(line):
  line = line.replace("\"", "").replace("\n", "").split(",")
  line = line[:-1] + [word for word in line[-1].split(" ") if word != ""]
  return line

def count_row(features, all_features):
  count = [1]  # y intercept
  for feature in all_features:
    count.append(features.count(feature))
  return count

def feature_scale(counts):
  mins = [1000] * (len(counts[0]) - 1)
  maxes = [-1000] * (len(counts[0]) - 1)
  sums = [0] * (len(counts[0]) - 1)
  for count in counts:
    for i in range(len(count) - 1):
      sums[i] += count[i + 1]
      if count[i + 1] < mins[i]:
        mins[i] = count[i + 1]
      if count[i + 1] > maxes[i]:
        maxes[i] = count[i + 1]
  means = [float(sums[i]) / len(counts) for i in range(len(sums))]
  ranges = [maxes[i] - mins[i] for i in range(len(maxes))]
  return means, ranges

def scale(count, means, ranges):
  scaled = [count[0]]
  for i in range(len(count) - 1):
    if ranges[i] > 0:
      scaled.append(float(count[i + 1] - means[i]) / ranges[i])
    else:
      scaled.append(0)
  return scaled

def rank(A, tol=1e-8):
  s = numpy.linalg.svd(numpy.array(A), compute_uv=0)
  return numpy.sum(numpy.where(s > tol, 1, 0))

def tfidf(counts, features):
  tf = []
  df = {}
  for count in counts:
    term_freq = {}
    size = 0
    for i in xrange(len(count[1:])):
      if count[i] > 0:
        term_freq[features[i]] = count[i + 1]
        size += count[i]
        if features[i] in df:
          df[features[i]] += 1
        else:
          df[features[i]] = 1
    tf.append((size, term_freq))
  sums = {}
  for (size, term_freq) in tf:
    for feature, count in term_freq.items():
      tfidf_val = float(count) / size * math.log(float(len(counts) - 1) / df[feature])
      if feature in sums:
        sums[feature] += tfidf_val
      else:
        sums[feature] = tfidf_val
  return sums

def linear_scoring(line, all_features, means, ranges, x):
  return (numpy.array(scale(count_row(feature_row(line), all_features), means, ranges)) * x).sum()

def feature_counts(all_features, features):
  all_features_dict = dict((feature, 0) for feature in all_features)
  for feature in features:
    for x in feature:
      all_features_dict[x] += 1
  return sorted(all_features_dict.iteritems(), key=lambda x: -x[1])

def get_limited_instance(instance, features):
  limited_instance = {}
  for feature in instance:
    if feature in features:
      if feature in limited_instance:
        limited_instance[feature] += 1
      else:
        limited_instance[feature] = 1
  return limited_instance

def nearest_score(instance, neighbor):
  score = 0
  for feature in instance:
    if feature in neighbor:
      score += instance[feature] - neighbor[feature]
    else:
      score += instance[feature]
  for feature in neighbor:
    if feature in instance:
      score += neighbor[feature] - instance[feature]
    else:
      score += neighbor[feature]
  return score

def nearest_scoring(line, neighbors, features):
  instance_row = feature_row(line)
  instance = get_limited_instance(instance_row, features)
  nearest_neighbor = (9998, None)
  next_nearest_neighbor = (9999, None)
  for neighbor in neighbors:
    score = nearest_score(instance, neighbor[1])
    if score <= (len(instance_row) + sum(y for (x, y) in neighbor[1].iteritems())) / 2:
      if score < nearest_neighbor[0]:
        next_nearest_neighbor = nearest_neighbor
        nearest_neighbor = (score, neighbor)
      elif score < next_nearest_neighbor[0]:
        next_nearest_neighbor = (score, neighbor)
  if nearest_neighbor[1] == None and next_nearest_neighbor[1] == None:
    return None
  elif next_nearest_neighbor[1] == None:
    return nearest_neighbor[1][0]
  else:
    return (nearest_neighbor[1][0] + next_nearest_neighbor[1][0]) / 2

def main():
  # Break input into easy-to-use Python lists and list-of-lists.
  lines = open("ratings.txt").readlines()
  scores = []
  features = []
  for line in lines:
    score, feature = break_row(line)
    scores.append(score)
    features.append(feature)
  all_features = set()
  for s in features:
    for t in s:
      all_features.add(t)

  # Now build regressor matrix for linear regression.
  # all_features_ordered = list(all_features)
  # counts = [count_row(feature, all_features_ordered) for feature in features]
  # counts_rank = rank(counts)

  # Build sum-of-term frequency inverse document frequency counts for each term in
  # the input. Was going to try to use this to remove overtly overused/underused
  # terms to reduce feature space (since we have more features than examples...)
  # and make linear regression less underdetermined and thus avoid the need for
  # regularization. Doesn't seem to work the way I thought it did though.
  # tfidfs = tfidf(counts, all_features_ordered).items()
  # tfidfs.sort(key = lambda x: -x[1])

  # Scale regressor matrix into common ranges (-1 <= x <= 1) using mean
  # normalization. Really only helpful for gradient descent, but I thought it
  # would avoid my underdetermined problems. Nope.
  # means, ranges = feature_scale(counts)
  # coeffs = numpy.array([scale(count, means, ranges) for count in counts])
  # scores = numpy.array(scores)

  # Solve the least squares problem using numpy.
  #x, _, _, _ = numpy.linalg.lstsq(coeffs, scores)

  # Build kNN model with filtering.
  # Assume last entries in features are newest, take the latest 125 of them,
  # encode only the most-common 50 features in a dictionary from word->count.
  # When scoring, do an average of 2 nearest neighbors search with filtering (if
  # no neighbor matches more than 50% of features, don't return a score). Metric
  # is number of words not matched.
  limited_features = [x for (x, y) in feature_counts(all_features, features)[:50]]
  instances = features[-125:]
  instances_score = scores[-125:]
  limited_instances = []
  for i in range(125):
    limited_instances.append((
        instances_score[i], get_limited_instance(instances[i], limited_features)))

  # Run my test cases.

  # Linear regression.
  # test = "\"ari.wilson\",\"mmmm.hm_-_tv_central_forum\",\"file  parks and recreation 420 hdtv lol mp4 thread  parks and recreation   season 4\""
  # print linear_scoring(test, all_features_ordered, means, ranges, x)

  # kNN.
  new_lines = [line.split(",", 1) for line in lines[:50]]
  total_error = 0
  for (score, line) in new_lines:
    nearest_score = nearest_scoring(line, limited_instances, limited_features)
    print float(score), nearest_score
    total_error += abs(float(score) - nearest_score)
  print total_error

if __name__ == "__main__":
  main()
