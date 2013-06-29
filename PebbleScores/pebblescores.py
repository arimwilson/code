# TODO(ariw): Some sort of registration interface so others can add their own
# games?

import webapp2 as webapp

from google.appengine.api import memcache
from google.appengine.ext import db
from google.appengine.ext.webapp.util import run_wsgi_app

class Game(db.Model):
  name = db.StringProperty(required = True)
  key = db.BlobProperty(required = True)

class User(db.Model):
  name = db.StringProperty(required = True)
  last_ip_address = db.StringProperty(required = True)

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

class SubmitHandler(webapp.RequestHandler):
  def post(self):
    game = getGame(self.request.get("game"))
    user = getUser(self.request.get("username"))
    if not user:
      pass

class ListHandler(webapp.RequestHandler):
  def get(self):
    # Get the top 20 scores by game.
    game = getGame(self.request.get("game"))

app = webapp.WSGIApplication([
    ('/submit', SubmitHandler),
    ('/list', ListHandler),
  ])
