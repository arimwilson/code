# TODO(ariw): Some sort of registration interface so others can add their own
# games?

import hashlib
import hmac
import json
import webapp2 as webapp

from google.appengine.api import memcache
from google.appengine.ext import db
from google.appengine.ext.webapp.util import run_wsgi_app

class Game(db.Model):
  name = db.StringProperty(required = True)
  mac_key = db.BlobProperty(required = True)

class User(db.Model):
  name = db.StringProperty(required = True)
  ip_address = db.StringProperty(required = True)
  num_zero_games = db.IntegerProperty()

class HighScore(db.Model):
  # TODO(ariw): Should game and user be reference properties?
  game = db.StringProperty(required = True)
  user = db.StringProperty(required = True)
  score = db.IntegerProperty(required = True)
  created = db.DateTimeProperty(required = True, auto_now_add = True)

# Get a list of entities of type model with property=filter from either memcache
# or the datastore, updating memcache if we have to go to the datastore.
def getEntities(model, property, filter):
  cache_key = "%s,%s:%s" % (model, property, filter)
  entities = memcache.get(cache_key)
  if entities:
    return entities
  query = eval(model).all()
  query.filter("%s =" % property, filter)
  entities = [entity for entity in query]
  if not entities:
    return entities
  memcache.add(cache_key, entities)
  return entities

def getUser(username):
  users = getEntities("User", "name", username)
  if not users:
    return
  return users[0]

def getGame(game):
  games = getEntities("Game", "name", game)
  if not games:
    return
  return games[0]

def getMac(game, score, mac_key):
  # TODO(ariw): Probably want to have a nonce (or use a timestamp) to prevent
  # replay attacks...
  message = "%s%d" % (game, score)
  return hmac.new(mac_key, message, hashlib.sha256).hexdigest()

class SubmitHandler(webapp.RequestHandler):
  def post(self):
    request = json.loads(self.request.body)
    username = self.request.headers["X-PEBBLE-ID"]
    user = getUser(username)
    if not user:
      user = User(name = username, ip_address=self.request.remote_addr,
                  num_zero_games = 0)
      user.put()
    score = request["2"]
    # Don't store a highscore entry if the score was 0.
    if score == 0:
      user.num_zero_games += 1
      user.put()
      return

    game = getGame(request["1"])
    if (not game or
        getMac(str(game.name), score, game.mac_key) != request["3"]):
      self.error(403)
      return
    highscore = HighScore(
        game = game.name, user = user.name, score = score)
    highscore.put()

_HIGHSCORE_HTML_TEMPLATE = """
  <li><b>%(highscore)s</b> - %(username)s"""

def highscoreHtml(highscore):
  return _HIGHSCORE_HTML_TEMPLATE % {
      "highscore": highscore.score, "username": highscore.user}

_HTML_TEMPLATE = """
  <!DOCTYPE HTML>
  <html lang="en">
  <head>
  <title>%(game)s Scores</title>
  </head>
  <body>
  <ol>
  %(list)s
  </ol>
  </body>
  </html>"""

class ListHandler(webapp.RequestHandler):
  def get(self):
    # Get the top 100 scores by game.
    query = HighScore.all()
    game = self.request.get("game")
    query.filter("game =", game)
    query.order("-score")
    highscores = query.fetch(100)
    highscores_html = [highscoreHtml(highscore) for highscore in highscores]
    self.response.out.write(
        _HTML_TEMPLATE % {"game": game,
                          "list": "".join(highscores_html)})

app = webapp.WSGIApplication([
    ('/submit', SubmitHandler),
    ('/list', ListHandler),
  ])
