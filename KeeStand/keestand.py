import logging
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
      if user.file_ref:
        self.response.out.write(str(user.file_ref.key()))
    else:  # New user.
      user = User(username=username, password_hash=password_hash)
      user.put()
    self.response.headers.add_header(
        "Set-Cookie",
        "username=%s:password_hash=%s" % (username, password_hash))

def AuthorizedUser(cookies):
  username = cookies["username"]
  password_hash = cookies["password_hash"]
  query = User.all()
  query.filter("username =", username)
  query.filter("password_hash =", password_hash)
  user = query.get()
  if not user:
    logging.error("Should never get here! File uploaded but username (%s) "
                  "or password_hash (%s) is wrong!" %
                  (username, password_hash))
  return user == None, user

class LoadHandler(blobstore_handlers.BlobstoreDownloadHandler):
  def post(self, resource):
    success, _ = AuthorizedUser(self.str_cookies)
    if not success:
      return
    resource = str(urllib.unquote(resource))
    blob_info = blobstore.BlobInfo.get(resource)
    self.send_blob(blob_info)

class SaveUrlHandler(webapp.RequestHandler):
  def post(self):
    success, _ = AuthorizedUser(self.str_cookies)
    if not success:
      return
    self.response.out.write(blobstore.create_upload_url("/script/save"))

class SaveHandler(blobstore_handlers.BlobstoreUploadHandler):
  def post(self):
    file_ref = self.get_uploads("file")[0]
    success, user = AuthorizedUser(self.str_cookies)
    if not success:
      file_ref.delete()
      return
    if user.file_ref:
      user.file_ref.delete()
    user.file_ref = self.get_uploads("file")[0]
    user.put()
    # TODO(ariw): Redirect?

def main():
  application = webapp.WSGIApplication([
      ('/script/login', LoginHandler),
      ('/script/load/([^/]+)?', LoadHandler),
      ('/script/save_url', SaveUrlHandler),
      ('/script/save', SaveHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()

