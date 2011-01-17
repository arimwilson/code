import base64
import logging
import os
import string

from google.appengine.ext import blobstore
from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

class User(db.Model):
  version = db.IntegerProperty(required = True)
  username = db.StringProperty(required = True)
  salt = db.StringProperty(required = True)
  password_hash = db.StringProperty(required = True)

# We can only store up to 1 megabyte of passwords per datastore entity. So we
# split up passwords into multiple chunks.
class PasswordChunk(db.Model):
  user = db.ReferenceProperty(User)
  chunk = db.TextProperty()

class SaltHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user:
      self.response.out.write(user.salt)
    else:
      self.response.out.write(base64.b64encode(os.urandom(16)))

class LoginHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    assert username
    password_hash = self.request.get("password_hash")
    assert password_hash
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user and password_hash != user.password_hash:  # Failed login.
      self.error(401)
      return
    elif user:  # Existing user, success.
      if user.passwordchunk_set:  # Existing data.
        for passwordchunk in user.passwordchunk_set:
          self.response.out.write(str(passwordchunk.chunk))
    else:  # New user.
      salt = self.request.get("salt")
      assert salt
      user = User(version = 1, username = username, salt = salt,
                  password_hash = password_hash)
      user.put()
    self.response.headers.add_header(
        "Set-Cookie",
        "session=%s.%s" % (
            base64.b64encode(username), password_hash))

def AuthorizedUser(cookies):
  session = cookies.get("session", "").split(".")
  username = base64.b64decode(session[0])
  password_hash = session[1]
  query = User.all()
  query.filter("username =", username)
  query.filter("password_hash =", password_hash)
  user = query.get()
  if not user:
    logging.error("Should never get here! Data received but username (%s) or "
                  "password_hash (%s) is wrong!" % (username, password_hash))
  return user <> None, user

# Split a large string into a list of chunks of size n.
def Split(string, n):
  split = []
  for i in xrange(0, len(string), n):
    split.append(string[i:i + n])
  return split

class SaveHandler(webapp.RequestHandler):
  def post(self):
    success, user = AuthorizedUser(self.request.cookies)
    if not success:
      self.error(401)
      return
    if user.passwordchunk_set:
      for passwordchunk in user.passwordchunk_set:
        passwordchunk.delete()
    # Since no characters in base64-encoded JSON are Unicode, we can store
    # exactly 1 << 20 characters in one entity property.
    for chunk in Split(self.request.get("passwords"), 1 << 20):
      PasswordChunk(user = user, chunk = db.Text(chunk)).put()

class DeleteAccountHandler(webapp.RequestHandler):
  def post(self):
    success, user = AuthorizedUser(self.request.cookies)
    if not success:
      self.error(401)
      return
    db.delete([user] + [chunk for chunk in user.passwordchunk_set])

def main():
  application = webapp.WSGIApplication([
      ('/script/salt', SaltHandler),
      ('/script/login', LoginHandler),
      ('/script/save', SaveHandler),
      ('/script/deleteaccount', DeleteAccountHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()

