#!/usr/bin/python
# Database:
#  See market.db.
#  TODO(ariw): Add min/max/step amounts to SellOffer table?
#
# Input:
#  Currency buying
#  Currency selling
#  User location
#  Amount of currency desired
#
# Output:
#  Ordered list of sellers and desired quantities matching currencies and
#  location sorted by online auction based on reviews and bid amount.

import argparse, sqlite3

def normalize(x, min, max):
  return (x - min) / (max - min)

def offer_score(bid, average_rating):
  # TODO(ariw): Normalize bid by dynamic min/max.
  MAX_BID = 12.3
  BID_WEIGHT = 0.4
  MIN_AVERAGE_RATING = 1.0
  MAX_AVERAGE_RATING = 5.0
  AVERAGE_RATING_WEIGHT = 0.6
  if average_rating is None:
    average_rating = MAX_AVERAGE_RATING / 2
  return (BID_WEIGHT * normalize(bid, 0, MAX_BID) +
          AVERAGE_RATING_WEIGHT * normalize(
              average_rating, MIN_AVERAGE_RATING, MAX_AVERAGE_RATING))

def match_buyer(currency_buying, currency_selling, user_location, quantity_sold,
                conn):
  # Determine average seller rating for the later auction.
  AVERAGE_RATING_SQL = """
  SELECT SellerId, AVG(Rating) FROM SellerReview GROUP BY SellerId;
  """
  average_ratings = {}
  for row in conn.execute(AVERAGE_RATING_SQL):
    average_ratings[row[0]] = row[1]
  # Get all matching offers.
  OFFER_FILTER_SQL = """
  SELECT SellerId, Contact, Bid
  FROM Seller JOIN
       SellerOffer USING (SellerId) JOIN
       SellerLocation USING (SellerId)
  WHERE CurrencySold = "%s" AND CurrencyBought = "%s" AND Location = "%s" AND
        QuantityBought = %d;
  """
  offers = []
  for row in conn.execute(OFFER_FILTER_SQL % (
      currency_buying, currency_selling, user_location, quantity_sold)):
    offers.append([row[1], row[2], row[0]])
  # Get scores for offers.
  scores = {}
  for offer in offers:
    scores[offer[2]] = offer_score(offer[1], average_ratings.get(offer[2]))
  print scores
  # Sort seller offers by score, ascending.
  offers.sort(key=lambda offer: scores[offer[2]], reverse=True)
  # Remove ids from offers before return.
  for offer in offers:
    offer.pop()
  return offers

if __name__ == "__main__":
  parser = argparse.ArgumentParser(
      description = "Bidding currency seller / user market, like MyMoncy.")
  parser.add_argument("currency_buying")
  parser.add_argument("currency_selling")
  parser.add_argument("user_location")
  parser.add_argument("quantity_sold")
  parser.add_argument("--database", default="market.db")
  args = parser.parse_args()

  print match_buyer(
      args.currency_buying, args.currency_selling, args.user_location,
      args.quantity_sold, sqlite3.connect(args.database))
