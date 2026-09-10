import sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
p = r"E:\PYTHON\CoreSwap\.tmp\perf-reg-260910-06\logs\os-r1.log"
raw = open(p, encoding="utf-8", errors="replace").read().splitlines()
MARK = "[STALLWATCH]"
dumps, cur = [], None
for l in raw:
    i = l.find(MARK)
    if i < 0: continue
    body = l[i+len(MARK):]
    if "=== thread dump" in body: cur = []; dumps.append(cur); continue
    if "=== end ===" in body: cur = None; continue
    if cur is not None: cur.append(body)
print("dumps:", len(dumps))
last = dumps[-1]
threads = []
for b in last:
    m = re.match(r'\s*"(.+?)" id=(\d+) state=(\w+)', b)
    if m: threads.append([m.group(1), m.group(3), []])
    elif threads: threads[-1][2].append(b.strip())
from collections import Counter
print("threads:", len(threads), dict(Counter(t[1] for t in threads)))
print("\n=== threads with key frames (wg.bench / chunk machinery / LockHelper) ===")
KEY = ("wg.bench", "LockHelper", "ChunkTaskScheduler", "ChunkStatus", "NoiseChunkGenerator", "ThreadedAnvilChunkStorage", "CompletableFuture", "ChunkMap", "ServerChunkManager", "Chunky", "worldgen")
for name, state, stack in threads:
    if any(any(k in f for k in KEY) for f in stack):
        print(f'\n## {name} state={state}')
        for f in stack[:22]:
            print("   ", f)
