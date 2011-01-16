import base64
import logging
import urllib

from google.appengine.ext import blobstore
from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

class User(db.Model):
  username = db.StringProperty(required = True)
  password_hash = db.StringProperty(required = True)
  passwords = db.TextProperty()

class LoginHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    password_hash = self.request.get("password_hash")
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user and password_hash != user.password_hash:  # Failed login.
      self.response.out.write("FAIL")
      return
    elif user:  # Existing user, success.
      if user.passwords:
        self.response.out.write(str(user.passwords))
    else:  # New user.
      user = User(username=username, password_hash=password_hash)
      user.put()
    self.response.headers.add_header(
        "Set-Cookie",
        "session=%s.%s" % (
            base64.b64encode(username), base64.b64encode(password_hash)))

def AuthorizedUser(cookies):
  session = [base64.b64decode(x) for x in cookies.get("session", "").split(".")]
  username, password_hash = session
  query = User.all()
  query.filter("username =", username)
  query.filter("password_hash =", password_hash)
  user = query.get()
  if not user:
    logging.error("Should never get here! File uploaded but username (%s) "
                  "or password_hash (%s) is wrong!" %
                  (username, password_hash))
  return user <> None, user

class SaveHandler(webapp.RequestHandler):
  def post(self):
    success, user = AuthorizedUser(self.request.cookies)
    if not success:
      return
    user.passwords = db.Text(self.request.get("passwords"))
    user.put()

def main():
  application = webapp.WSGIApplication([
      ('/script/login', LoginHandler),
      ('/script/save', SaveHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()

