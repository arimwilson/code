from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

class AddHandler(webapp.RequestHandler):
  def post(self):
    pass

class UpdateHandler(webapp.RequestHandler):
  def post(self):
    pass

def main()
  application = webapp.WSGIApplication([
      ('/add', AddHandler),
      ('/tasks/update', UpdateHandler),
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()
