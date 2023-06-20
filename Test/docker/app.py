# app.py
from flask import Flask, request
app = Flask(__name__)

@app.route('/')
def hello_world():
    return 'Hello, World! Port: ' + str(request.environ.get('SERVER_PORT'))

if __name__ == '__main__':
    app.run(host='0.0.0.0')
