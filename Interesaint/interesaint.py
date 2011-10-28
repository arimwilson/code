from google.appengine.ext import db
from google.appengine.ext import webapp
from google.appengine.ext.webapp.util import run_wsgi_app

def main():
  application = webapp.WSGIApplication([
    ])
  run_wsgi_app(application)

if __name__ == '__main__':
  main()

