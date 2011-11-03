from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

class Feed(db.Model):
  url = db.StringProperty(required = True)

class User(db.Model):
  username = db.StringProperty(required = True)

class Subscription(db.Model):
  user = db.ReferenceProperty(User, required = True)
  feed = db.ReferenceProperty(Feed, required = True)

class Item(db.Model):
  feed = db.ReferenceProperty(Feed, required = True)
  published = db.DateTimeProperty(required = True)
  retrieved = db.DateTimeProperty(required = True, auto_now_add = True)
  content = db.StringProperty(required = True)

class Rating(db.Model):
  user = db.ReferenceProperty(User, required = True)
  item = db.ReferenceProperty(Item, required = True)
  interesting = db.FloatProperty(required = True)

class AddHandler(webapp.RequestHandler):
  def post(self):
    pass

class UpdateHandler(webapp.RequestHandler):
  def post(self):
    pass

def main()
  application = webapp.WSGIApplication([
      ('/add', AddHandler),
      ('/tasks/update', UpdateHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()
