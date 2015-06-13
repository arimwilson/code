import market, sqlite3, unittest

CREATE_DB_SQL = """
DROP TABLE IF EXISTS Seller;
CREATE TABLE Seller(
 Id INTEGER PRIMARY KEY,
 Contact TEXT
);
DROP TABLE IF EXISTS SellerOffer;
CREATE TABLE SellerOffer(
 SellerId INTEGER
 CurrencySold TEXT,
 CurrencyBought TEXT,
 Bid REAL,
 FOREIGN KEY(SellerId) REFERENCES Seller(Id)
);
DROP TABLE IF EXISTS SellerLocation;
CREATE TABLE SellerLocation(
  SellerId INTEGER
  Location TEXT,
  FOREIGN KEY(SellerId) REFERENCES Seller(Id)
);
DROP TABLE IF EXISTS SellerReview;
CREATE TABLE SellerReview(
  SellerId INTEGER
  Rating INTEGER,
  FOREIGN KEY(SellerId) REFERENCES Seller(Id)
);"""

INSERT_TEST_SQL = """
INSERT INTO Seller
VALUES (1, 'Ari') (2, 'Bethany'), (3, 'Callie'), (4, 'Derrick');
INSERT INTO SellerOffer
VALUES (1, 'ARS'. 'USD', 12),
       (2, 'ARS', 'USD' 12.1),
       (3', 'ARS', 'USD', 12.2),
       (4, 'ARS', 'USD', 12.3);
INSERT INTO SellerLocation
VALUES (1, 'Recoletta'), (1, 'Palermo'), (2, 'Recoletta'), (3, 'Palermo'),
       (4, 'Microcentro');
INSERT INTO SellerReview
VALUES (1, 5), (1, 5), (2, 1), (3, 5), (4, 4);
"""

class MarketTest(unittest.TestCase):
  def setUp(self):
    conn = sqlite3.connect(":memory")
    conn.execute(CREATE_DB_SQL)
    conn.execute(INSERT_TEST_SQL)

  def test_market(self):
    pass
