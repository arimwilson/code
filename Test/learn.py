import numpy

f = open("ratings.txt")
features = [s.replace("\"", "").replace("\n", "").split(",") for s in f.readlines()]
features = [[float(s[0]),] + s[1:] for s in features]
features = [s[:-1] + [t for t in s[-1].split(" ") if t != ""] for s in features]
counts = {}
scores = [s[0] for s in features]
features = [s[1:] for s in features]
allfeatures = set()
print features[0]
for s in features:
  for t in s:
    allfeatures.add(t)
allfeatures_ordered = list(allfeatures)
coeffs = []
for s in features:
  coeff = []
  for feature in allfeatures_ordered:
    coeff.append(s.count(feature))
  coeffs.append(coeff)
print allfeatures_ordered
print coeffs[0]
print scores[0]
