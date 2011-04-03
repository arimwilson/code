# Find numbers with the most divisors in some range.

import math

num_divisors = []
for i in range(2, 1001):
  k = 0
  l = []
  for j in range(2, i):
    if j >= 50 and j <= 500 and i % j == 0:
      l.append(j)
      k += 1
  num_divisors.append((i, k, l))

print sorted(num_divisors, reverse=True, key=lambda (i, k, l): k)[:5]

