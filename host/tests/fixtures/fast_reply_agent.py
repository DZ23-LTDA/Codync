"""ACP agent that answers a prompt at once: one `session/update` with text, then the stop reason.

The point is not the answer but the clock: with an agent that replies immediately, whatever time passes between `send`
and the first entry in the transcript is the host's own overhead, not the model's.
"""

import json
import sys

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
    elif method == "session/prompt":
        session = params.get("sessionId", "s1")
        send({"method": "session/update", "params": {"sessionId": session, "update": {
            "sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "pong"}}}})
        send({"id": message["id"], "result": {"stopReason": "end_turn"}})
    elif method == "session/cancel":
        send({"id": message["id"], "result": {}})
    else:
        send({"id": message["id"], "error": {"code": -32601, "message": "not supported"}})
