# Try to recreate MyMoncy as a bidding currency seller / user market.
#
# "Database"::
#  Seller 1 ID
#  Location(s)
#  Min / max / step amounts
#  Review(s) (out of 5 stars)
#  Currency 1 ID / bid / min / max / step amounts
#  Currency 2 ...
#
#  Seller 2 ...
#
# Input:
#  Currency ID
#  User location
#  Amount of currency desired
#
# Output:
#  Ordered list of sellers matching location, currency desired sorted by online
#  auction based on reviews and bid amount.
