// Utility functions.

#ifndef COMMON_H
#define COMMON_H

#include "pebble_os.h"

const int kTimeZoneOffset = -7;  // PDT

int common_rand_seed = 0;
void common_srand(int seed) {
  common_rand_seed = seed;
}
int common_rand() {
  common_rand_seed = (((common_rand_seed * 214013L + 2531011L) >> 16) & 32767);
  return common_rand_seed;
}

// Get Unix time given PblTm.
unsigned int common_time() {
  PblTm time;
  get_time(&time);
  return
      ((0-kTimeZoneOffset)*3600) + // time zone offset
      time.tm_sec + // start with seconds
      time.tm_min*60 + // add minutes
      time.tm_hour*3600 + // add hours
      time.tm_yday*86400L + // add days
      (time.tm_year-70)*31536000L + // add years since 1970
      ((time.tm_year-69)/4)*86400L - // add a day after leap years, starting in 1973
      ((time.tm_year-1)/100)*86400L + // remove a leap day every 100 years, starting in 2001
      ((time.tm_year+299)/400)*86400L; // add a leap day back every 400 years, starting in 2001
}

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
    int next_strike = common_rand() % (kMaxShuffleInteger - i);
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

#endif  // COMMON_H
