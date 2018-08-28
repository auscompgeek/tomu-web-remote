from flask import Flask, jsonify, request, send_file
from flask_sockets import Sockets


app = Flask(__name__)
sockets = Sockets(app)
tomus = {}


@sockets.route('/ws-tomu')
def tomu_control_ws(ws):
    global last_message
    tomus[ws] = None
    while not ws.closed:
        tomus[ws] = ws.receive()
    del tomus[ws]


@app.route('/')
def index():
    return send_file('index.html')


@app.route('/states')
def states():
    return jsonify(list(tomus.values()))


@app.route('/set', methods=['POST'])
def set_state():
    count = 0
    state = request.form['state']
    for ws in tomus:
        ws.send(state)
        count += 1
    return str(count)


if __name__ == "__main__":
    from gevent import pywsgi
    from geventwebsocket.handler import WebSocketHandler
    server = pywsgi.WSGIServer(('', 5000), app, handler_class=WebSocketHandler)
    server.serve_forever()
