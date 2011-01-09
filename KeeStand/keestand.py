import wsgiref.handlers

from google.appengine.ext import webapp

class LoginHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    password = self.request.get("password")
    self.response.out.write("<html><title>KeeStand</title><body>%s %s</body></html>" % (username, password))

class EditHandler(webapp.RequestHandler):
  def post(self):
    pass

def main():
  application = webapp.WSGIApplication([
      ('/script/login', LoginHandler),
      ('/script/edit', EditHandler),
    ])
  wsgiref.handlers.CGIHandler().run(application)

if __name__ == '__main__':
  main()
