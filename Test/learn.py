import numpy

def break_row(line):
  line = line.split(",", 1)
  return float(line[0]), feature_row(line[1])

def feature_row(line):
  line = line.replace("\"", "").replace("\n", "").split(",")
  line = line[:-1] + [word for word in line[-1].split(" ") if word != ""]
  return line

def coeff_row(features, all_features):
  coeff = [1]  # y intercept
  for feature in all_features:
    coeff.append(features.count(feature))
  return coeff

def feature_scale(coeffs):
  mins = [1000] * (len(coeffs[0]) - 1)
  maxes = [-1000] * (len(coeffs[0]) - 1)
  sums = [0] * (len(coeffs[0]) - 1)
  for coeff in coeffs:
    for i in range(len(coeff) - 1):
      sums[i] += coeff[i + 1]
      if coeff[i + 1] < mins[i]:
        mins[i] = coeff[i + 1]
      if coeff[i + 1] > maxes[i]:
        maxes[i] = coeff[i + 1]
  means = [float(sums[i]) / len(coeffs) for i in range(len(sums))]
  ranges = [maxes[i] - mins[i] for i in range(len(maxes))]
  return means, ranges

def scale(coeff, means, ranges):
  scaled = [coeff[0]]
  for i in range(len(coeff) - 1):
    if ranges[i] > 0:
      scaled.append(float(coeff[i + 1] - means[i]) / ranges[i])
    else:
      scaled.append(0)
  return scaled

def scoring(line, all_features, mins, maxes, x):
  return (numpy.array(scale(coeff_row(feature_row(line), all_features), mins, maxes)) * x).sum()

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
all_features_ordered = list(all_features)
coeffs = [coeff_row(feature, all_features_ordered) for feature in features]
means, ranges = feature_scale(coeffs)
coeffs = numpy.array([scale(coeff, means, ranges) for coeff in coeffs])
scores = numpy.array(scores)
x, residues, rank, s = numpy.linalg.lstsq(coeffs, scores)
test = "\"ari.wilson\",\"mmmm.hm_-_tv_central_forum\",\"file  project accessory s01e05 beach blanket blingo ws dsr xvid  ny2  avi thread  project accessory season 1\""
print scoring(test, all_features_ordered, means, ranges, x)
print x
