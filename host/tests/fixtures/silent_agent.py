"""ACP agent that accepts a prompt and then goes silent: no `session/update`, no response to `session/prompt`, and it
keeps reading stdin so the host cannot tell the process died. This is the stuck turn the watchdog exists for."""

import json
import sys
import time

open_sessions = set()


def send(value):
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", **value}) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    message = json.loads(line)
    method, params = message.get("method"), message.get("params") or {}
    if "id" not in message or method is None:
        continue
    if method == "initialize":
        send({"id": message["id"], "result": {"protocolVersion": 1, "agentCapabilities": {}}})
    elif method == "session/new":
        open_sessions.add("s1")
        send({"id": message["id"], "result": {"sessionId": "s1"}})
    elif method == "session/cancel":
        # A real agent would answer something here; this one stays silent on purpose, so the host has to fall back to
        # killing the process once the cancel grace is over.
        time.sleep(3600)
    elif method == "session/prompt":
        # Accept the turn and never answer it: the turn is now stuck, with no update and no stop reason.
        continue
    else:
        send({"id": message["id"], "error": {"code": -32601, "message": "not supported"}})
