import urllib

from google.appengine.ext import blobstore
from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp import blobstore_handlers
from google.appengine.ext.webapp.util import run_wsgi_app

class User(db.Model):
  username = db.StringProperty(required = True)
  password_hash = db.StringProperty(required = True)
  file_ref = blobstore.BlobReferenceProperty()

class LoginHandler(webapp.RequestHandler):
  def get(self):
    username = self.request.get("username")
    password_hash = self.request.get("password_hash")
    query = User.all()
    query.filter("username =", username)
    user = query.get()
    if user and password_hash != user.password_hash:
      self.response.out.write("FAIL")
      return
    elif user:
      if user.file_ref:
        self.response.out.write(str(user.file_ref.key()))
    else:
      user = User(username=username, password_hash=password_hash)
      user.put()
    self.response.headers.add_header(
        "Set-Cookie", "username=%s:password_hash=%s" % (username, password_hash))

class ServeHandler(blobstore_handlers.BlobstoreDownloadHandler):
  def get(self, resource):
    resource = str(urllib.unquote(resource))
    blob_info = blobstore.BlobInfo.get(resource)
    self.send_blob(blob_info)

class SaveHandler(blobstore_handlers.BlobstoreUploadHandler):
  def post(self):
    pass

def main():
  application = webapp.WSGIApplication([
      ('/script/login', LoginHandler),
      ('/script/serve/([^/]+)?', ServeHandler),
      ('/script/save', SaveHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()
