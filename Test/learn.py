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

def feature_scale(countss):
  mins = [1000] * (len(counts[0]) - 1)
  maxes = [-1000] * (len(countss[0]) - 1)
  sums = [0] * (len(countss[0]) - 1)
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

def scoring(line, all_features, means, ranges, x):
  return (numpy.array(scale(count_row(feature_row(line), all_features), means, ranges)) * x).sum()

def word_counts(all_features):
  all_features_dict = dict((feature, 0) for feature in all_features)
  for feature in features:
    for x in feature:
      all_features_dict[x] += 1
  print sorted(all_features_dict.iteritems(), key=lambda x: -x[1])

lines = open("ratings.txt").readlines()
scores = []
features = []
for line in lines:
  print line
  score, feature = break_row(line)
  scores.append(score)
  features.append(feature)
all_features = set()
for s in features:
  for t in s:
    all_features.add(t)
all_features_ordered = list(all_features)
counts = [count_row(feature, all_features_ordered) for feature in features]
counts_rank = rank(counts)
tfidfs = tfidf(counts, all_features_ordered).items()
tfidfs.sort(key = lambda x: -x[1])
print tfidfs
means, ranges = feature_scale(counts)
coeffs = numpy.array([scale(count, means, ranges) for count in counts])
scores = numpy.array(scores)
x, _, _, _ = numpy.linalg.lstsq(coeffs, scores)
test = "\"ari.wilson\",\"mmmm.hm_-_tv_central_forum\",\"file  parks and recreation 417 hdtv lol mp4 thread  parks and recreation   season 4\""
print scoring(test, all_features_ordered, means, ranges, x)
