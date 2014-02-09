# Utility functions to manipulate PebbleScores datastore.

import pebblescores

from google.appengine.ext import db

# Delete any extra users with the same name.
def clearDuplicateUsers():
  q = db.Query(pebblescores.User, projection=('name',))
  q.order('name')
  oldUsername = "FAKE"
  entities_to_delete = []
  i = 0
  for user in q:
    if i % 50 == 0:
      print user.name
    if user.name == oldUsername:
      print "deleting %s" % user.name
      entities_to_delete.append(user)
      if len(entities_to_delete) % 20 == 0:
        db.delete(entities_to_delete)
        entities_to_delete = []
    else:
      oldUsername = user.name
    i = i + 1
  db.delete(entities_to_delete)
