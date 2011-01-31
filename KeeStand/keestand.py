import base64
import gzip
import logging
import pickle
import os
import re
import StringIO

from django.utils import simplejson as json
from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

class User(db.Model):
  # Used to upgrade legacy user authentication / encryption methods if code
  # changes.
  version = db.IntegerProperty(required = True)
  username = db.StringProperty(required = True)
  # Salt needed for client to generate authentication hash.
  salt = db.BlobProperty(required = True)
  # Password hash used for authentication.
  password_hash = db.ByteStringProperty(required = True)
  # When this user was created.
  created = db.DateTimeProperty(required = True, auto_now_add = True)
  # When this user / user's password was last modified.
  last_modified = db.DateTimeProperty(required = True, auto_now = True)

# We can only store up to 1 megabyte of passwords per datastore entity. So we
# split up passwords into multiple chunks.
class PasswordChunk(db.Model):
  user = db.ReferenceProperty(User, required = True)
  index = db.IntegerProperty(required = True)
  chunk = db.BlobProperty(required = True)

class SaltHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user:
      self.response.out.write(base64.standard_b64encode(user.salt))
    else:
      self.response.out.write(base64.standard_b64encode(os.urandom(16)))

# Convert gzipped pickled Python dictionary back to JSON.
def Decode(string):
  string = gzip.GzipFile(fileobj=StringIO.StringIO(string)).read()
  # TODO(ariw): WTF why can't I depickle?!
  dictionary = pickle.loads(string)
  for (key, value) in dictionary.iteritems():
    dictionary[key] = base64.standard_b64encode(value)
  output = json.dumps(dictionary)
  # SJCL wants invalid JSON which we hack around here.
  output = re.sub("([\{,]) ?\"(.*?)\": ", r"\1\2:", output)
  # Python puts in dumb \ characters in base64-decoded strings, so let's get rid
  # of them.
  output = output.replace("\\", "")
  return output

class LoginHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    assert username
    password_hash = self.request.get("password_hash")
    assert password_hash
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user and password_hash != base64.standard_b64encode(user.password_hash):
      # Failed login.
      self.error(401)
      return
    elif user:  # Existing user, success.
      chunks = sorted(user.passwordchunk_set, key = lambda chunk: chunk.index)
      passwords = "".join([chunk.chunk for chunk in chunks])
    if passwords:  # Existing data.
      self.response.out.write(Decode(passwords))
    else:  # New user.
      salt = self.request.get("salt")
      assert salt
      user = User(
          version = 1, username = username,
          salt = db.Blob(base64.standard_b64decode(salt)),
          password_hash = db.ByteString(
              base64.standard_b64decode(password_hash)))
      user.put()
    self.response.headers.add_header(
        "Set-Cookie",
        "session=%s.%s" % (
            base64.standard_b64encode(username), password_hash))

def AuthorizedUser(cookies):
  session = cookies.get("session", "").split(".")
  username = base64.standard_b64decode(session[0])
  password_hash = session[1]
  query = User.all()
  query.filter("username =", username)
  query.filter("password_hash =",
               db.ByteString(base64.standard_b64decode(password_hash)))
  user = query.get()
  if not user:
    logging.error("Should never get here! Data received but username (%s) or "
                  "password_hash (%s) is wrong!" % (username, password_hash))
  return user <> None, user

# Convert JSON to gzipped pickled Python dictionary.
def Encode(string):
  # SJCL produces invalid JSON which we hack around here.
  string = re.sub(r"([\{,])(.*?):", "\\1\"\\2\":", string)
  dictionary = json.loads(string)
  for (key, value) in dictionary.iteritems():
    dictionary[key] = base64.standard_b64decode(value)
  pickled = pickle.dumps(dictionary, pickle.HIGHEST_PROTOCOL)
  output = StringIO.StringIO()
  gzip.GzipFile(fileobj=output, mode="wb").write(pickled)
  return output.getvalue()

# Split a large string into a list of chunks of at most size n.
def Split(string, n):
  split = []
  for i in xrange(0, len(string), n):
    split.append(string[i:i + n])
  return split

# Save new password file as one datastore transaction.
def Save(user, password_chunks, old_password_chunks):
  for index, chunk in enumerate(password_chunks):
    PasswordChunk(parent = user, user = user, index = index,
                  chunk = db.Blob(chunk)).put()
  db.delete([chunk for chunk in old_password_chunks])
  # Update last_modified.
  user.put()

class SaveHandler(webapp.RequestHandler):
  def post(self):
    success, user = AuthorizedUser(self.request.cookies)
    if not success:
      self.error(401)
      return
    passwords = Encode(self.request.get("passwords"))
    # Can store at least 10 ** 6 bytes in one entity property.
    password_chunks = Split(passwords, 10 ** 6)
    old_password_chunks = [chunk for chunk in user.passwordchunk_set]
    db.run_in_transaction(Save, user, password_chunks, old_password_chunks)

def DeleteAccount(password_chunks, user):
  db.delete([chunk for chunk in password_chunks] + [user])

class DeleteAccountHandler(webapp.RequestHandler):
  def post(self):
    success, user = AuthorizedUser(self.request.cookies)
    if not success:
      self.error(401)
      return
    password_chunks = [chunk for chunk in user.passwordchunk_set]
    db.run_in_transaction(DeleteAccount, password_chunks, user)

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
