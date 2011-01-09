import wsgiref.handlers

from google.appengine.ext import webapp

class ViewHandler(webapp.RequestHandler):
  def post(self):
    username = self.request.get("username")
    password = self.request.get("password")
    self.response.out.write("<html><title>KeeStand</title><body>%s %s</body></html>" % (username, password))

class EditHandler(webapp.RequestHandler):
  def get(self):
    pass

  def post(self):
    pass

def main():
  application = webapp.WSGIApplication([
      ('/view', ViewHandler),
      ('/edit', EditHandler),
    ], debug=True)
  wsgiref.handlers.CGIHandler().run(application)

if __name__ == '__main__':
  main()
