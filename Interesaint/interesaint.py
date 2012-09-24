import datetime
import feedparser
import gzip
import json
import logging
import math
import pickle
import re
import StringIO
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
  username = db.StringProperty(required = True)

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
  created = db.DateTimeProperty(auto_now_add = True)
  interesting = db.FloatProperty()

class PredictionModel(db.Model):
  user = db.ReferenceProperty(User, required = True)
  model = db.BlobProperty()

def getLearningItem(item):
  feed_title = "_".join(item.feed.title.lower().split())
  title = [item.title.lower()]
  for token in " ,;:-.!?\"/()[]":
    title = sum((t.split(token) for t in title), [])
  return [feed_title] + title

def getLimitedItem(learning_item, feature_counts):
  limited_item = {}
  for feature in learning_item:
    if not feature_counts or feature in feature_counts:
      if feature in limited_item:
        limited_item[feature] += 1
      else:
        limited_item[feature] = 1
  return limited_item

def getNearestScore(limited_item, neighbor):
  score = 0
  for feature in limited_item:
    if feature in neighbor:
      score += limited_item[feature] - neighbor[feature]
    else:
      score += limited_item[feature]
  for feature in neighbor:
    if feature in limited_item:
      score += neighbor[feature] - limited_item[feature]
    else:
      score += neighbor[feature]
  return score

def predict(prediction_model, item):
  if not prediction_model:
    return None
  # Build a score based on the two nearest neighbors subject to filtering.
  limited_item = getLimitedItem(getLearningItem(item), None)
  nearest_neighbor = (9998, None)
  next_nearest_neighbor = (9999, None)
  for neighbor in prediction_model:
    score = getNearestScore(limited_item, neighbor[1])
    if score <= 7:
      if score < nearest_neighbor[0]:
        next_nearest_neighbor = nearest_neighbor
        nearest_neighbor = (score, neighbor)
      elif score < next_nearest_neighbor[0]:
        next_nearest_neighbor = (score, neighbor)
  if nearest_neighbor[1] == None and next_nearest_neighbor[1] == None:
    return None
  elif next_nearest_neighbor[1] == None:
    return nearest_neighbor[1][0]
  else:
    return (nearest_neighbor[1][0] + next_nearest_neighbor[1][0]) / 2

def getUser(username):
  user = memcache.get("User,user:" + username)
  if user:
    return user
  query = User.all()
  query.filter("username =", username)
  user = query.get()
  if not user:
    return
  memcache.add("User,user:" + username, user)
  return user

def getSubscriptions(user):
  subscriptions = memcache.get("Subscriptions,user:" + user.username)
  if subscriptions:
    return subscriptions
  query = Subscription.all()
  query.filter("user =", user)
  subscriptions = [subscription for subscription in query]
  if not subscriptions:
    return []
  memcache.add("Subscriptions,user:" + user.username, subscriptions)
  return subscriptions

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

def getPredictionModel(user):
  prediction_model = memcache.get("PredictionModel,user:" + user.username)
  if prediction_model:
    return prediction_model
  query = PredictionModel.all()
  query.filter("user =", user)
  prediction_model = query.get()
  if not prediction_model:
    return
  memcache.add("PredictionModel,user:" + user.username, prediction_model)
  return prediction_model

# Get the score of this item related to star rating, predicted rating and date.
def getMagic(item):
  rating = 0.5  # Default rating is 2.5 stars.
  # Have to override if you have a real rating. Normalize from [0.2, 1] to
  # [0, 1]
  if item.predicted_interesting:
    rating = 5/4 * (item.predicted_interesting - 0.2)
  if item.interesting:
    rating = 5/4 * (item.interesting - 0.2)
  # Take time since item was updated, normalize from [0, inf] to [0, 1] in the
  # last month (2592000 seconds), and invert.
  diff = (datetime.datetime.utcnow() - item.updated).total_seconds()
  time_rating = 1 - diff / 2592000
  alpha, beta = 4, 1  # Weights on score versus time.
  return alpha * rating + beta * time_rating

def getItems(user, subscriptions, page, prediction_model, magic):
  feeds = tuple(subscription.feed for subscription in subscriptions)
  query = Item.all()
  query.filter("feed IN", feeds).order("-updated")
  items_to_display = 20
  if magic:
    items_to_fetch = 100
  else:
    items_to_fetch = 20
  page_offset = items_to_fetch * (items_to_display * page / items_to_fetch)
  items = query.fetch(items_to_fetch, page_offset)
  # Batch up lookups for ratings.
  ratings = []
  # TODO(ariw): Instead of getting ratings this way, cross-reference with items
  # somehow or sort by *item* updated rather than rating created?
  for i in xrange(0, len(items), 30):
    query = Rating.all()
    query.filter("user =", user)
    query.filter("item IN", tuple(items[i:i+30]))
    ratings.extend(query.fetch(items_to_fetch))
  for item in items:
    item.interesting = None
  for feed in feeds:
    for item in items:
      if item.feed.key() == feed.key():
        item.feed_title = feed.title
  for rating in ratings:
    for item in items:
      if rating.item.key() == item.key():
        item.interesting = rating.interesting
        break
  for item in items:
    item.predicted_interesting = predict(prediction_model, item)
  page_index = items_to_display * page % items_to_fetch
  if magic:
    items = sorted(items, key=getMagic, reverse=True)
  return items[page_index:page_index + items_to_display]


# Get most recommended items for a user.
class ItemHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      self.error(403)
      return
    prediction_model = getPredictionModel(user)
    if prediction_model:
      prediction_model = pickle.loads(gzip.GzipFile(
          fileobj=StringIO.StringIO(prediction_model.model)).read())
    # TODO(ariw): Gotta be a better way to do this.
    magic = True if self.request.get("magic") == "true" else False
    items = getItems(
        user, getSubscriptions(user), int(self.request.get("page")),
        prediction_model, magic)
    self.response.out.write(json.dumps([getPublicItem(item) for item in items]))

# Get a list of user subscriptions.
class SubscriptionHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      self.error(403)
      return
    subscriptions = getSubscriptions(user)
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

# Add a new subscription for a user.
class AddHandler(webapp2.RequestHandler):
  def post(self):
    username = users.get_current_user().nickname()
    user = getUser(username)
    if not user:
      user = User(username = username)
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
      memcache.delete("Subscriptions,user:" + user.username)
      subscription.put()

class RemoveHandler(webapp2.RequestHandler):
  def post(self):
    id = long(self.request.get("id"))
    subscription = Subscription.get_by_id(id)
    memcache.delete("Subscriptions,user:" +
        getUser(users.get_current_user().nickname()).username)
    db.delete(subscription)
    # TODO(ariw): Remove feeds and/or ratings?

# Rate some item.
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
      rating = Rating(user = user, item = item, interesting = interesting)
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

# Return the logout link.
class LogoutLinkHandler(webapp2.RequestHandler):
  def post(self):
    self.response.out.write(users.create_logout_url("/"))

# Get latest items from feeds.
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

# Remove old items and ratings to clear up space.
class CleanHandler(webapp2.RequestHandler):
  def get(self):
    items_to_delete = []
    ratings_to_delete = []
    for item in Item.all():
      # 6 months so I can actually build up a classification history!
      if item.retrieved < datetime.datetime.now() - datetime.timedelta(6 * 30):
        items_to_delete.append(item)
        ratings_to_delete += item.rating_set
    db.delete(items_to_delete)
    db.delete(ratings_to_delete)

def getFeatureCounts(learning_items, limit=-1):
  feature_counts = {}
  for learning_item in learning_items:
    for feature in learning_item:
      if feature in feature_counts:
        feature_counts[feature] += 1
      else:
        feature_counts[feature] = 1
  feature_counts = sorted(
      feature_counts.iteritems(), key=lambda x: -x[1])
  if limit < 0:
    return dict(feature_counts)
  else:
    return dict(feature_counts[:limit])

# Generates a model to predict user ratings.
class LearnHandler(webapp2.RequestHandler):
  def get(self):
    # For each user, build a nearest neighbor model based on the most recent
    # ratings' feeds and titles.
    NUM_ITEMS = 100
    NUM_FEATURES = 100
    query = User.all()
    ratings = []
    limited_items = []
    for user in query:
      query = Rating.all()
      query.filter("user =", user).order("-created")
      ratings = query.fetch(NUM_ITEMS)
      learning_items = [getLearningItem(rating.item) for rating in ratings]
      feature_counts = getFeatureCounts(learning_items, NUM_FEATURES)
      limited_items = []
      for i in range(NUM_ITEMS):
        limited_items.append(
            (ratings[i].interesting,
             getLimitedItem(learning_items[i], feature_counts)))
      pickled = pickle.dumps(limited_items, 2)
      output = StringIO.StringIO()
      gzip.GzipFile(fileobj=output, mode="wb").write(pickled)
      query = PredictionModel.all()
      query.filter("user =", user)
      prediction_model = query.get()
      if not prediction_model:
        prediction_model = PredictionModel(
            user = user, model = output.getvalue())
      else:
        prediction_model.model = output.getvalue()
      memcache.delete("PredictionModel,user:" + user.username)
      prediction_model.put()

# List ratings and their items. Used for training learning hypotheses outside
# the bounds of AppEngine.
class RatingsListingHandler(webapp2.RequestHandler):
  def get(self):
    query = User.all()
    ratings = []
    for user in query:
      query = Rating.all()
      query.filter("user =", user)
      username = "_".join(user.username.lower().split())
      for rating in query:
        if not rating.interesting:
          continue
        learning_item = getLearningItem(rating.item)
        ratings.append("%f,\"%s\",\"%s\",\"%s\"" % (
            rating.interesting, username, learning_item[0],
            " ".join(learning_item[1:])))
    self.response.out.write("\n".join(ratings))

# List items (up to 100) to test learning hypotheses outside the bounds of
# AppEngine.
class ItemsListingHandler(webapp2.RequestHandler):
  def get(self):
    # TODO(ariw): Fill out.
    pass

app = webapp2.WSGIApplication([
    ('/script/items', ItemHandler),
    ('/script/subscriptions', SubscriptionHandler),
    ('/script/add', AddHandler),
    ('/script/remove', RemoveHandler),
    ('/script/rate', RateHandler),
    ('/script/logoutlink', LogoutLinkHandler),
    ('/tasks/update', UpdateHandler),
    ('/tasks/clean', CleanHandler),
    ('/tasks/learn', LearnHandler),
    ('/admin/ratings', RatingsListingHandler),
    ('/admin/items', ItemsListingHandler),
  ])
