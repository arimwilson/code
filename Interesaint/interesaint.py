import feedparser

from google.appengine.api import users
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
  # TODO(ariw): This should probably use blobstore.
  content = db.StringProperty(required = True)

class Rating(db.Model):
  user = db.ReferenceProperty(User, required = True)
  item = db.ReferenceProperty(Item, required = True)
  interesting = db.FloatProperty(required = True)

class AddHandler(webapp.RequestHandler):
  def post(self):
    query = User.all()
    username = users.get_current_user().nickname()
    query.filter("username =", username)
    user = query.get()
    if not user:
      user = User(username = username)
      user.put()

    query = Feed.all()
    url = self.request.get("url")
    query.filter("url =", url)
    feed = query.get()
    if not feed:
      feed = Feed(url = url)
      feed.put()

    query = Subscription.all()
    query.filter("user = ", user)
    query.filter("feed = ", feed)
    subscription = query.get()
    if not subscription:
      subscription = Subscription(user = user, feed = feed)
      subscription.put()

class UpdateHandler(webapp.RequestHandler):
  def post(self):
    pass

# TODO(ariw): Probably need some sort of clear handler to keep data sizes down.

def main():
  application = webapp.WSGIApplication([
      ('/add', AddHandler),
      ('/tasks/update', UpdateHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()
