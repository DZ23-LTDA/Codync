#!/usr/bin/env python3
"""How well Claude bots' prompt cache holds, read from Claude Code's own session logs.

Each session file in ~/.claude/projects is one conversation; each API call in it is counted once.
Usage: tools/memory-eval/cache.py [folder-name-part ...]   (default: the bots of ~/.codync and ~/.codync-dev)
"""
import glob
import json
import os
import sys
from collections import Counter
from datetime import datetime
from statistics import mean, median

ROOT = os.path.expanduser('~/.claude/projects')
LIFE = 3600  # Claude Code writes 1-hour cache entries.


def calls(path, seen):
    """(read, total input, 1h writes, writes, model, seconds since the previous call) per API call."""
    prev = None
    for line in open(path, errors='replace'):
        try:
            rec = json.loads(line)
        except ValueError:
            continue
        msg = rec.get('message') if isinstance(rec, dict) else None
        if not isinstance(msg, dict) or msg.get('role') != 'assistant' or 'usage' not in msg:
            continue
        key = msg.get('id') or rec.get('uuid')
        if key in seen:  # a forked session repeats its parent's calls
            continue
        seen.add(key)
        u = msg['usage']
        read = u.get('cache_read_input_tokens') or 0
        write = u.get('cache_creation_input_tokens') or 0
        total = read + write + (u.get('input_tokens') or 0)
        if total == 0:
            continue
        ts = datetime.fromisoformat(rec['timestamp'].replace('Z', '+00:00'))
        gap = (ts - prev).total_seconds() if prev else None
        prev = ts
        w1h = (u.get('cache_creation') or {}).get('ephemeral_1h_input_tokens') or 0
        yield read, total, w1h, write, msg.get('model', ''), gap


def cost(read, total, w1h, write, model):
    """Input cost at API list multipliers: read 0.1x (0.05x on the 5.5 models), 5-minute write 1.25x, 1-hour 2x."""
    return total - read - write + 1.25 * (write - w1h) + 2 * w1h + (0.05 if '-5-5' in model else 0.1) * read


def main():
    parts = sys.argv[1:] or ['-codync-bots-', '-codync-dev-bots-']
    sessions, causes, sums = [], Counter(), Counter()
    for folder in glob.glob(os.path.join(ROOT, '*')):
        if not any(p in os.path.basename(folder) for p in parts):
            continue
        seen = set()
        for path in glob.glob(os.path.join(folder, '*.jsonl')):
            rows = list(calls(path, seen))
            if rows:
                sessions.append(rows)
    if not sessions:
        sys.exit('no sessions found')
    every, warm, per_every, per_warm = [], [], [], []
    for rows in sessions:
        hot = [r for r in rows if r[5] is not None and r[5] <= LIFE]
        every += rows
        warm += hot
        per_every.append(sum(r[0] for r in rows) / sum(r[1] for r in rows))
        if hot:
            per_warm.append(sum(r[0] for r in hot) / sum(r[1] for r in hot))
        for read, total, w1h, write, model, gap in rows:
            sums['w1h'] += w1h
            sums['write'] += write
            sums['cost'] += cost(read, total, w1h, write, model)
            if read >= total / 2:
                continue
            causes['first call' if gap is None else 'after the cache expired' if gap > LIFE else 'mid-session miss'] += 1

    def pct(x):
        return f'{100 * x:5.1f}%'

    def line(name, rows, per):
        by_tokens = sum(r[0] for r in rows) / sum(r[1] for r in rows)
        return (f'{name:<22} {pct(by_tokens):>9}  {pct(mean(r[0] / r[1] for r in rows)):>8}  '
                f'{pct(mean(per)):>11}  {pct(median(per)):>9}')

    total = sum(r[1] for r in every)
    print(f'{len(sessions)} conversations, {len(every)} calls, {total / 1e6:.2f}M input tokens\n')
    print(f"{'hit rate':<22} {'by tokens':>9}  {'per call':>8}  {'per session':>11}  {'median':>9}")
    print(line('in a conversation', warm, per_warm))
    print(line('every call', every, per_every))
    print('\n"in a conversation" leaves out each first call and calls more than an hour after the previous one.')
    print(f'cold calls (under half read from cache): {dict(causes)}')
    print(f"1-hour writes: {pct(sums['w1h'] / max(sums['write'], 1))} of cache writes; "
          f"input cost vs. uncached: {pct(sums['cost'] / total)}")


main()
