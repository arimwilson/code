# TODO(ariw): Some sort of registration interface so others can add their own
# games?

import md5
import webapp2 as webapp

from google.appengine.api import memcache
from google.appengine.ext import db
from google.appengine.ext.webapp.util import run_wsgi_app

class Game(db.Model):
  name = db.StringProperty(required = True)
  key = db.BlobProperty(required = True)

class User(db.Model):
  name = db.StringProperty(required = True)
  ip_address = db.StringProperty(required = True)

class HighScore(db.Model):
  game = db.ReferenceProperty(Game, required = True)
  user = db.ReferenceProperty(User, required = True)
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

def getUser(name):
  return getEntities("User", "name", user)

def getGame(game):
  return getEntities("Game", "name", game)

def getMac(request, key):
  # TODO(ariw): HMAC instead of MD5?
  message = request.get("game") + request.get("username") +
            str(request.get("score") + key
  return md5.new(message).hexdigest()

class SubmitHandler(webapp.RequestHandler):
  def post(self):
    game = getGame(self.request.get("game"))
    if not game or getMac(self.request, game.key) != self.request.get("mac"):
      self.error(403)
      return
    username = self.request.get("username")
    user = getUser(username)
    if not user:
      user = User(name = username, ip_address=self.request.remote_addr)
      user.put()
    highscore = HighScore(
        game = game, user = user, score = self.request.get("score"))
    highscore.put()

_HIGHSCORE_HTML_TEMPLATE = """
  <li><b>%(highscore)s</b> - %(username)s
  """

def highscoreHtml(highscore):
  return _HIGHSCORE_HTML_TEMPLATE % {
      "highscore": highscore.score, "username": highscore.user.name}

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
    # Get the top 20 scores by game.
    game = getGame(self.request.get("game"))
    query = HighScore.All()
    query.filter("game =", game)
    query.order("-score")
    highscores = query.fetch(20)
    highscores_html = [highscoreHtml(highscore) for highscore in highscores]
    self.response.out.write(
        _HTML_TEMPLATE % {"game": game.name, "list": "".join(highscores_html)})

app = webapp.WSGIApplication([
    ('/submit', SubmitHandler),
    ('/list', ListHandler),
  ])
