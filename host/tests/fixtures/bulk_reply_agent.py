"""ACP agent that answers one prompt with a long reply, in many chunks.

The gate asks for the heavy screens (a long conversation) and for the host's behaviour when a turn writes a lot.
One prompt here becomes `CHUNKS` text updates of `CHUNK_WORDS` words each, so a single turn fills a transcript the
way a long agent answer does -- without a model in the way.

The host folds consecutive chunks into one message, so the number of *entries* a turn produces is the host's
choice, not this fixture's; `--chunks` says how much text it sends.
"""

import json
import os
import sys

CHUNKS = int(os.environ.get("BULK_CHUNKS", "400"))
CHUNK_WORDS = int(os.environ.get("BULK_WORDS", "12"))
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
        for index in range(CHUNKS):
            words = " ".join(f"palavra{index}-{word}" for word in range(CHUNK_WORDS))
            separator = "\n" if index % 20 == 19 else " "
            send({"method": "session/update", "params": {"sessionId": session, "update": {
                "sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": words + separator}}}})
        send({"method": "session/update", "params": {"sessionId": session, "update": {
            "sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "FIM-DA-RESPOSTA"}}}})
        send({"id": message["id"], "result": {"stopReason": "end_turn"}})
    elif method == "session/cancel":
        send({"id": message["id"], "result": {}})
    else:
        send({"id": message["id"], "error": {"code": -32601, "message": "not supported"}})
