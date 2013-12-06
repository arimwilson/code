// Utility functions.

#ifndef COMMON_H
#define COMMON_H

int common_mod(int a, int b) {
  int r = a % b;
  return r < 0? r + b : r;
}

const int kMaxShuffleInteger = 6;  // kLineSegments
// Return a length n random shuffle of the integers from [0,
// kMaxShuffleInteger). Assumes n <= kMaxShuffleInteger and shuffle's size is >=
// n.
void common_shuffle_integers(int n, int* shuffle) {
  bool struck_integers[kMaxShuffleInteger];
  for (int i = 0; i < kMaxShuffleInteger; ++i) {
    struck_integers[i] = false;
  }
  for (int i = 0; i < n; ++i) {
    int next_strike = rand() % (kMaxShuffleInteger - i);
    int j = 0, k = 0;
    for (; j <= next_strike; ++j) {
      while (struck_integers[j + k]) ++k;
    }
    struck_integers[j - 1 + k] = true;
    shuffle[i] = j - 1 + k;
  }
}

// Ascending insertion sort.
void common_insertion_sort(int* array, int size) {
  for (int i = 1; i < size; ++i) {
    for (int j = i; j > 0 && array[j] < array[j - 1]; --j) {
      int temp = array[j];
      array[j] = array[j - 1];
      array[j - 1] = temp;
    }
  }
}

int16_t common_min(int16_t a, int16_t b) {
  return (a < b)? a: b;
}

int16_t common_max(int16_t a, int16_t b) {
  return (a >= b)? a: b;
}

#endif  // COMMON_H
