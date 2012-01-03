import numpy

def break_row(line):
  line = line.split(",", 1)
  return float(line[0]), feature_row(line[1])

def feature_row(line):
  line = line.replace("\"", "").replace("\n", "").split(",")
  line = line[:-1] + [word for word in line[-1].split(" ") if word != ""]
  return line

def coeff_row(features, all_features):
  coeff = []
  for feature in all_features:
    coeff.append(features.count(feature))
  return coeff

def scoring(line, all_features, x):
  return (numpy.array(coeff_row(feature_row(line), all_features)) * x).sum()

lines = open("ratings.txt").readlines()
scores =[]
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
coeffs = numpy.array(coeffs)
scores = numpy.array(scores)
x, residues, rank, s = numpy.linalg.lstsq(coeffs, scores)
test = "\"ari.wilson\",\"mmmm.hm_-_tv_central_forum\",\"file  project accessory s01e05 beach blanket blingo ws dsr xvid  ny2  avi thread  project accessory season 1\""
print scoring(test, all_features_ordered, x)
