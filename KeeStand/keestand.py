from google.appengine.ext import blobstore
from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

class User(db.Model):
  username = db.StringProperty(required = True)
  password_hash = db.StringProperty(required = True)
  file_key = blobstore.BlobReferenceProperty()

class LoginHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    password_hash = self.request.get("password")
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user and password_hash <> user.password_hash:
      pass
    elif not user:
      user = User()
      user.username = username
      user.password_hash = password_hash
      user.put()
    self.response.headers["Set-Cookie"] = "username:%s" % username

class SaveHandler(webapp.RequestHandler):
  def post(self):
    pass

def main():
  application = webapp.WSGIApplication([
      ('/script/login', LoginHandler),
      ('/script/save', SaveHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()
