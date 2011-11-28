import datetime
import feedparser
import logging

from django.utils import simplejson as json
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
  retrieved = db.DateTimeProperty(required = True, auto_now_add = True)
  published = db.DateTimeProperty()
  updated = db.DateTimeProperty()
  title = db.StringProperty()
  # TODO(ariw): This should probably use blobstore.
  content = db.TextProperty()

class Rating(db.Model):
  user = db.ReferenceProperty(User, required = True)
  item = db.ReferenceProperty(Item, required = True)
  interesting = db.FloatProperty(required = True)

class ItemHandler(webapp.RequestHandler):
  def post(self):
    query = User.all()
    username = users.get_current_user().nickname()
    query.filter("username =", username)
    user = query.get()
    if not user:
      return
    query = Subscription.all()
    query.filter("user =", user)
    feeds = [subscription.feed for subscription in query]
    query = Item.all()
    query.filter("feed IN", tuple(feeds)).order("-updated")
    items = query.fetch(20)
    self.response.out.write(json.dumps(
        [{"title": item.title, "content": item.content} for item in items]))
    # TODO(ariw): We should really cache at least everything before the item
    # retrieval part here. Should be minimal data size.


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
    # Make sure this feed is okay before adding it.
    if feedparser.parse(url).bozo:
      self.error(400)
    query.filter("url =", url)
    feed = query.get()
    if not feed:
      feed = Feed(url = url)
      feed.put()

    query = Subscription.all()
    query.filter("user = ", user).filter("feed =", feed)
    subscription = query.get()
    if not subscription:
      subscription = Subscription(user = user, feed = feed)
      subscription.put()

def getFirstPresent(entry, tags):
  for tag in tags:
    if tag in entry:
      return entry[tag]
  return None

def getDateTime(time):
  if not time:
    return None
  return datetime.datetime(*time[:-3])

class UpdateHandler(webapp.RequestHandler):
  def get(self):
    for feed in Feed.all():
      query = Item.all()
      query.filter("feed =", feed).order("-updated")
      last_item = query.get()
      parsed_feed = feedparser.parse(feed.url)
      for entry in parsed_feed.entries:
        updated = getDateTime(getFirstPresent(entry, ["updated_parsed"]))
        if (last_item and last_item.updated and updated and
            last_item.updated >= updated):
          break
        published = getDateTime(getFirstPresent(entry, ["published_parsed"]))
        title = getFirstPresent(entry, ["title"])
        content = getFirstPresent(entry, ["content", "description"])
        logging.info(content)
        item = Item(feed = feed, published = published, updated = updated,
                    title = title, content = content)
        item.put()

# TODO(ariw): Probably need some sort of clear handler to keep data sizes down.

def main():
  application = webapp.WSGIApplication([
      ('/items', ItemHandler),
      ('/add', AddHandler),
      ('/tasks/update', UpdateHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()
