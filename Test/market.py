#!/usr/bin/python
# Database:
#  See market.db.
#  TODO(ariw): Add min/max/step amounts to SellOffer table?
#
# Input:
#  Currency buying
#  Currency selling
#  User location
#d  Amount of currency desired
#
# Output:
#  Ordered list of sellers matching currencies, location, and currency received
#  sorted by online auction based on reviews and bid amount.

import argparse, sqlite3

if __name__ == "__main__":
  parser = argparse.ArgumentParser(
      description = "Bidding currency seller / user market, like MyMoncy.")
  parser.add_argument("currency_id")
  parser.add_argument("user_location")
  parser.add_argument("amount_of_currency")
  parser.add_argument("--database", default="market.db")
  args = parser.parse_args()

  conn = sqlite3.connect(args.database)
