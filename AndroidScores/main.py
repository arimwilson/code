import wsgiref.handlers

from google.appengine.ext import webapp

class MainHandler(webapp.RequestHandler):
  def get(self):
    pass

def main():
  application = webapp.WSGIApplication([('/', MainHandler)],
                                       debug=True)
  wsgiref.handlers.CGIHandler().run(application)

if __name__ == '__main__':
  main()
