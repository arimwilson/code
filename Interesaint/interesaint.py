import datetime
import feedparser
import json
import logging
import re
import webapp2

from google.appengine.api import memcache
from google.appengine.api import urlfetch
from google.appengine.api import users
from google.appengine.ext import db

class Feed(db.Model):
  url = db.StringProperty(required = True)
  title = db.TextProperty()
  last_retrieved = db.DateTimeProperty()

class User(db.Model):
  username = db.TextProperty(required = True)

class Subscription(db.Model):
  user = db.ReferenceProperty(User, required = True)
  feed = db.ReferenceProperty(Feed, required = True)

class Item(db.Model):
  feed = db.ReferenceProperty(Feed, required = True)
  retrieved = db.DateTimeProperty(required = True, auto_now_add = True)
  published = db.DateTimeProperty()
  updated = db.DateTimeProperty()
  title = db.TextProperty()
  url = db.StringProperty()
  # TODO(ariw): This should probably use blobstore.
  content = db.TextProperty()
  comments = db.TextProperty()

class Rating(db.Model):
  user = db.ReferenceProperty(User, required = True)
  item = db.ReferenceProperty(Item, required = True)
  created = db.DateTimeProperty()
  interesting = db.FloatProperty()
  predicted_interesting = db.FloatProperty()

def getUser(username):
  user = memcache.get(username)
  if user:
    return user
  query = User.all()
  query.filter("username =", username)
  user = query.get()
  if not user:
    return
  memcache.add(username, user)
  return user

def getPublicDate(date):
  if date:
    return str(date) + " GMT"
  else:
    return None

# Convert from datastore entity to item to be sent to user.
def getPublicItem(item):
  return { "id": item.key().id(), "rating": item.interesting,
           "predicted_rating": item.predicted_interesting,
           "feed_title": item.feed_title,
           "retrieved": getPublicDate(item.retrieved),
           "published": getPublicDate(item.published),
           "updated": getPublicDate(item.updated), "title": item.title,
           "url": item.url, "content": item.content, "comments": item.comments }

class ItemHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      self.error(403)
      return
    query = Subscription.all()
    query.filter("user =", user)
    feeds = [subscription.feed for subscription in query]
    query = Item.all()
    query.filter("feed IN", tuple(feeds)).order("-updated")
    items = query.fetch(20, 20 * int(self.request.get("page")))
    query = Rating.all()
    query.filter("user =", user)
    query.filter("item IN", tuple(items))
    ratings = query.fetch(20)
    for item in items:
      item.interesting = None
      item.predicted_interesting = None
    for feed in feeds:
      for item in items:
        if item.feed.key() == feed.key():
          item.feed_title = feed.title
    for rating in ratings:
      for item in items:
        if rating.item.key() == item.key():
          item.interesting = rating.interesting
          item.predicted_interesting = rating.predicted_interesting
          break
    self.response.out.write(json.dumps(
        [getPublicItem(item) for item in items]))
    # TODO(ariw): We should really cache at least everything before the item
    # retrieval part here. Should be minimal data size.

class SubscriptionHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      self.error(403)
      return
    query = Subscription.all()
    query.filter("user =", user)
    subscriptions = [subscription for subscription in query]
    query = Feed.all()
    feeds = Feed.get_by_id(
        [subscription.feed.key().id() for subscription in subscriptions])
    for feed in feeds:
      for subscription in subscriptions:
        if subscription.feed.key() == feed.key():
          subscription.title = feed.title
    self.response.out.write(json.dumps(
        [{"id": subscription.key().id(), "title": subscription.title} for
          subscription in subscriptions]))

def logResponse(response):
  logging.info("status_code: %d, headers: '%s', content: '%s'" %
               (response.status_code, str(response.headers), response.content))

class AddHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      user = User(username = users)
      user.put()

    query = Feed.all()
    url = self.request.get("url")
    # Make sure this feed is okay before adding it.
    parsed_feed = feedparser.parse(url)
    if parsed_feed.bozo:
      logging.error("Cannot parse feed %s with error %s" %
                    (url, str(parsed_feed.bozo_exception)))
      logResponse(urlfetch.fetch(url))
      self.error(400)
      return
    query.filter("url =", url)
    feed = query.get()
    if not feed:
      title = getFirstPresent(parsed_feed.feed, ["title"])
      feed = Feed(url = url, title = title)
      feed.put()

    query = Subscription.all()
    query.filter("user = ", user).filter("feed =", feed)
    subscription = query.get()
    if not subscription:
      subscription = Subscription(user = user, feed = feed)
      subscription.put()

class RemoveHandler(webapp2.RequestHandler):
  def post(self):
   subscription = Subscription.get_by_id(long(self.request.get("id")))
   db.delete(subscription)
   # TODO(ariw): Remove feeds and/or ratings?

class RateHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      self.error(403)
      return
    item = Item.get_by_id(long(self.request.get("id")))
    interesting = float(self.request.get("rating"))
    query = Rating.all()
    query.filter("user =", user).filter("item =", item)
    rating = query.get()
    if not rating:
      rating = Rating(user = user, item = item, interesting = interesting,
                      created = datetime.datetime.utcnow())
    else:
      rating.interesting = interesting
    rating.put()

# From dictionary entry, get the value that corresponds to the first present
# key from tags, or None.
def getFirstPresent(entry, tags):
  for tag in tags:
    if tag in entry:
      return entry[tag]
  return None

# Convert feedparser time to datetime.
def getDateTime(time):
  if not time:
    return None
  return datetime.datetime(*time[:-3])

class UpdateHandler(webapp2.RequestHandler):
  def get(self):
    for feed in Feed.all():
      query = Item.all()
      query.filter("feed =", feed).order("-updated")
      last_item = query.get()
      parsed_feed = feedparser.parse(feed.url)
      if parsed_feed.bozo:
        logging.error("Cannot parse feed %s with error %s" %
                      (feed.url, str(parsed_feed.bozo_exception)))
        logResponse(urlfetch.fetch(feed.url))
        continue
      feed.last_retrieved = datetime.datetime.utcnow()
      feed.put()
      # TODO(ariw): Add predicted ratings here.
      for entry in parsed_feed.entries:
        updated = getDateTime(getFirstPresent(entry, ["updated_parsed"]))
        title = getFirstPresent(entry, ["title"])
        if (last_item and last_item.updated and updated and
            last_item.updated >= updated):
          continue
        published = getDateTime(getFirstPresent(entry, ["published_parsed"]))
        url = getFirstPresent(entry, ["link"])
        # TODO(ariw): Fix to deal with weirdness with multiple contents.
        content = getFirstPresent(entry, ["content", "description"])
        comments = getFirstPresent(entry, ["comments"])
        item = Item(feed = feed, published = published, updated = updated,
                    title = title, url = url, content = content,
                    comments = comments)
        item.put()

class CleanHandler(webapp2.RequestHandler):
  def get(self):
    items_to_delete = []
    ratings_to_delete = []
    for item in Item.all():
      # 1 month so I can actually build up a classification history!
      if item.retrieved < datetime.datetime.now() - datetime.timedelta(30):
        items_to_delete.append(item)
        ratings_to_delete += item.rating_set
    db.delete(items_to_delete)
    db.delete(ratings_to_delete)

class RatingsHandler(webapp2.RequestHandler):
  def get(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      self.error(403)
      return
    query = Rating.all()
    query.filter("user =", user)
    username = "_".join(username.lower().split())
    for rating in query:
      if not rating.interesting:
        continue
      feed_title = "_".join(rating.item.feed.title.lower().split())
      split_title = [rating.item.title.lower()]
      for token in " ,;:-.!?\"/()[]":
        split_title = sum((s.split(token) for s in split_title), [])
      title = " ".join(split_title)
      self.response.out.write("%f,\"%s\",\"%s\",\"%s\"\n" % (
          rating.interesting, username, feed_title, title))

app = webapp2.WSGIApplication([
    ('/script/items', ItemHandler),
    ('/script/subscriptions', SubscriptionHandler),
    ('/script/add', AddHandler),
    ('/script/remove', RemoveHandler),
    ('/script/rate', RateHandler),
    ('/tasks/update', UpdateHandler),
    ('/tasks/clean', CleanHandler),
    # Temporary; used for training Google Prediction API.
    ('/script/ratings', RatingsHandler),
  ])
