// Utility functions.

#ifndef COMMON_H
#define COMMON_H

#include "pebble_os.h"

const int kTimeZoneOffset = -7;  // PDT

// Get Unix time given PblTm.
unsigned int unix_time(PblTm* current_time) {
  return
      ((0-kTimeZoneOffset)*3600) + // current_time zone offset
      current_time->tm_sec + // start with seconds
      current_time->tm_min*60 + // add minutes
      current_time->tm_hour*3600 + // add hours
      current_time->tm_yday*86400 + // add days
      (current_time->tm_year-70)*31536000 + // add years since 1970
      ((current_time->tm_year-69)/4)*86400 - // add a day after leap years, starting in 1973
      ((current_time->tm_year-1)/100)*86400 + // remove a leap day every 100 years, starting in 2001
      ((current_time->tm_year+299)/400)*86400; // add a leap day back every 400 years, starting in 2001
}

#endif  // COMMON_H
