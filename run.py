# This is a quick script for running Bevy in a webworker, it will build everything needed and
# create an output directory, containing the assets that need to be served. The script will then
# start a Python HTTP server.
#
# If you need support with your webapp, get in touch via contact@allwright.io and tell me about
# your project.

import http.server
import os
import signal
import socketserver
import subprocess

PORT = 3000

# build project
subprocess.run([
    "cargo",
    "build",
    "--release"
])

# create the output directory and symlink index.html and reset.css
if not os.path.isdir('output'):
    os.mkdir('output')
if not os.path.islink('output/index.html'):
    os.symlink('../index.html', 'output/index.html')
if not os.path.islink('output/reset.css'):
    os.symlink('../reset.css', 'output/reset.css')

# generate bindings
print('Generating bindings')
subprocess.run([
    "wasm-bindgen",
    "target/wasm32-unknown-unknown/debug/main.wasm",
    "--out-dir",
    "output",
    "--target",
    "web",
    "--no-typescript"
])
subprocess.run([
    "wasm-bindgen",
    "target/wasm32-unknown-unknown/debug/worker.wasm",
    "--out-dir",
    "output",
    "--target",
    "web",
    "--no-typescript"
])

print('Optimising WebAssembly')
subprocess.run([
    "wasm-opt",
    "-O1",
    "--enable-bulk-memory",
    "--output",
    "output/main_bg.wasm",
    "output/main_bg.wasm"
])
subprocess.run([
    "wasm-opt",
    "-O1",
    "--enable-bulk-memory",
    "--output",
    "output/worker_bg.wasm",
    "output/worker_bg.wasm"
])

# patch worker.js to start automatically (not needed when using a bundler)
with open('output/worker.js', 'r') as worker_js:
    worker_js_content = worker_js.read()
worker_js_content = \
    worker_js_content.replace('export default __wbg_init;', '__wbg_init();')
with open('output/worker.js', 'w') as worker_js:
    worker_js.write(worker_js_content)

# start web server
class RequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory='output', **kwargs)
server = socketserver.TCPServer(('localhost', PORT), RequestHandler)
signal.signal(signal.SIGINT,
    lambda _signal, _frame: setattr(server, '_BaseServer__shutdown_request', True))
print('Serving on http://localhost:{}/'.format(PORT))
server.serve_forever()
