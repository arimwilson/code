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

def match_buyer(currency_id, user_location, amount_of_currency, conn):
  pass

if __name__ == "__main__":
  parser = argparse.ArgumentParser(
      description = "Bidding currency seller / user market, like MyMoncy.")
  parser.add_argument("currency_id")
  parser.add_argument("user_location")
  parser.add_argument("amount_of_currency")
  parser.add_argument("--database", default="market.db")
  args = parser.parse_args()

  print match_buyer(
      args.currency_id, args.user_location, args.amount_of_currency,
      sqlite3.connect(args.database))
