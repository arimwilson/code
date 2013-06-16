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

#endif  // COMMON_H
