# TODO(ariw): Some sort of registration interface so others can add their own
# games?

import base64
import hashlib
import hmac
import json
import logging
import os
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
  # TODO(ariw): Should game, user, and account_token be reference properties?
  game = db.StringProperty(required = True)
  user = db.StringProperty(required = True)
  account_token = db.StringProperty(required = True)
  score = db.IntegerProperty(required = True)
  created = db.DateTimeProperty(required = True, auto_now_add = True)

class NonceHandler(webapp.RequestHandler):
  def post(self):
    nonce = base64.standard_b64encode(os.urandom(16))
    # Use memcache to transiently store the nonce until the client sends us
    # their score.
    client = memcache.Client()
    client.set(nonce, True)
    self.response.out.write(json.dumps({"nonce": nonce}))

def getEntitiesCacheKey(model, property, filter):
  return "%s,%s:%s" % (model, property, filter)

# Get a list of entities of type model with property=filter from either memcache
# or the datastore, updating memcache if we have to go to the datastore.
def getEntities(model, property, filter):
  cache_key = getEntitiesCacheKey(model, property, filter)
  client = memcache.Client()
  entities = client.get(cache_key)
  if entities:
    return entities
  query = eval(model).all()
  query.filter("%s =" % property, filter)
  entities = [entity for entity in query]
  if not entities:
    return entities
  client.add(cache_key, entities)
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

def validateNonce(nonce):
  client = memcache.Client()
  validated_nonce = client.gets(nonce)
  # We compare-and-set the nonce to False here to prevent it from being reused
  # by other score submissions.
  return validated_nonce and client.cas(nonce, False, 1)

def getMac(game, score, nonce, mac_key):
  if nonce:
    message = "%s%d%s" % (game, score, nonce)
  else:
    message = "%s%d" % (game, score)
  return hmac.new(mac_key, message, hashlib.sha256).hexdigest()

class SubmitHandler(webapp.RequestHandler):
  # Get the user, get the game, verify the nonce, verify the hash, store the
  # score :).
  def post(self):
    request = json.loads(self.request.body)
    username = self.request.headers["X-PEBBLE-ID"]
    user = getUser(username)
    if not user:
      user = User(name = username, ip_address=self.request.remote_addr,
                  num_zero_games = 0)
      user.put()
    game = getGame(request["name"])
    if not game:
      logging.error("Game %s not found." % request["name"])
      self.error(403)
      return
    # TODO(ariw): Nonce is not in legacy Falldown client code. This security
    # hole should be removed soon.
    nonce = request.get("nonce", None)
    if nonce and not validateNonce(nonce):
      logging.error("Nonce %s not found." % nonce)
      self.error(403)
      return
    score = request["score"]
    mac = getMac(str(game.name), score, nonce, game.mac_key)
    if  mac != request["mac"]:
      logging.error(
          "Server MAC %s did not equal request MAC %s." % (mac, request["mac"]))
      self.error(403)
      return

    # Don't store a highscore entry if the score was 0.
    if score == 0:
      user.num_zero_games += 1
      # Have to invalidate user cache since we're changing the underlying user.
      memcache.delete(getEntitiesCacheKey("User", "name", username))
      user.put()
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
    ('/nonce', NonceHandler),
    ('/submit', SubmitHandler),
    ('/list', ListHandler),
  ])
