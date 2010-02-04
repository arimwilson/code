import wsgiref.handlers

from google.appengine.ext import webapp

class SubmitHandler(webapp.RequestHandler):
  def get(self):
    pass

  def post(self):
    pass


class ListHandler(webapp.RequestHandler):
  def get(self):
    pass

  def post(self):
    pass

def main():
  application = webapp.WSGIApplication([
      ('/submit', SubmitHandler),
      ('/list', ListHandler),
    ], debug=True)
  wsgiref.handlers.CGIHandler().run(application)

if __name__ == '__main__':
  main()
