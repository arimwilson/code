# Database:
'''
create table Seller(
 Id INTEGER PRIMARY KEY,
 Contact TEXT
);
create table SellerOffer(
 SellerId INTEGER
 CurrencySold TEXT,
 CurrencyBought TEXT,
 Bid REAL,
 FOREIGN KEY(SellerId) REFERENCES Seller(Id)
);
create table SellerLocation(
  SellerId INTEGER
  Location TEXT,
  FOREIGN KEY(SellerId) REFERENCES Seller(Id)
);
create table SellerReview(
  SellerId INTEGER
  Rating INTEGER,
  FOREIGN KEY(SellerId) REFERENCES Seller(Id)
);
'''
# TODO(ariw): Add min/max/step amounts to SellOffer table?
#
# Input:
#  Currency buying
#  Currency selling
#  User location
#  Amount of currency desired
#
# Output:
#  Ordered list of sellers matching currencies, location, and currency received
#  sorted by online auction based on reviews and bid amount.

import argparse
import sqlite3

parser = argparse.ArgumentParser(
  description = "Bidding currency seller / user market, like MyMoncy.")
parser.add_argument("currency_id")
parser.add_argument("user_location")
parser.add_argument("amount_of_currency")
parser.add_argument("--database", default="market.db")
args = parser.parse_args()

conn = sqlite3.connect(args.database)
