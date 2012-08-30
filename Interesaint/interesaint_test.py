from interesaint import *

import unittest

from google.appengine.api import memcache
from google.appengine.ext import db
from google.appengine.ext import testbed

class InteresaintTestCase(unittest.TestCase):
  def setUp(self):
    self.testbed = testbed.Testbed()
    self.testbed.activate()
    self.testbed.init_datastore_v3_stub()
    self.testbed.init_memcache_stub()

  def tearDown(self):
    self.testbed.deactivate()

  def testGetUserDb(self):
    user = User(username = u"test")
    user.put()
    query = User.all()
    user = query.get()
    self.assertEqual(user.username, getUser(u"test").username)

  def testGetUserMemcache(self):
    user = User(username = u"test")
    memcache.add("User,user:test", user)
    self.assertEqual(user.username, getUser(u"test").username)

if __name__ == "__main__":
  unittest.main()
