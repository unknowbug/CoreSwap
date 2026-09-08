# 260908-09 P3 C3-①: parse product tree diff 1.20.1 vs 1.21.6
# Compare PARSED JSON trees (not raw text, #12) for:
#   - noise_settings/overworld.json (whole tree, esp. noise_router)
#   - density_function/**/*.json registry
# Report structural diffs by path; classify "key table only" vs semantic.
import sys, os, json
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

A = r"E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen"
B = r"E:\PYTHON\CoreSwap\versions\1.21.6\data\worldgen\data\minecraft\worldgen"

def load(p):
    with open(p, encoding="utf-8") as f:
        return json.load(f)

diffs = []
def cmp(a, b, path):
    if type(a) is not type(b):
        diffs.append((path, "TYPE", type(a).__name__, type(b).__name__)); return
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a: diffs.append((path + "/" + k, "ADDED", "-", ""))
            elif k not in b: diffs.append((path + "/" + k, "REMOVED", "", "-"))
            else: cmp(a[k], b[k], path + "/" + k)
    elif isinstance(a, list):
        if len(a) != len(b):
            diffs.append((path, "LEN", len(a), len(b)))
        for i, (x, y) in enumerate(zip(a, b)):
            cmp(x, y, f"{path}[{i}]")
    else:
        if a != b:
            diffs.append((path, "VALUE", a, b))

# 1) noise_settings/overworld.json
cmp(load(os.path.join(A, "noise_settings", "overworld.json")),
    load(os.path.join(B, "noise_settings", "overworld.json")), "noise_settings/overworld")
# also nether for later
n_diffs_before = len(diffs)
cmp(load(os.path.join(A, "noise_settings", "nether.json")),
    load(os.path.join(B, "noise_settings", "nether.json")), "noise_settings/nether")
nether_start = n_diffs_before

# 2) density_function registry (overworld subdir like transpiler: density_function/overworld)
def collect(root):
    reg = {}
    for dirpath, _, files in os.walk(root):
        for fn in files:
            if fn.endswith(".json"):
                full = os.path.join(dirpath, fn)
                rel = os.path.relpath(full, root).replace("\\", "/")[:-5]
                reg[rel] = load(full)
    return reg

dfa = collect(os.path.join(A, "density_function", "overworld"))
dfb = collect(os.path.join(B, "density_function", "overworld"))
only_a = sorted(set(dfa) - set(dfb)); only_b = sorted(set(dfb) - set(dfa))
tree_diffs = []
for k in sorted(set(dfa) & set(dfb)):
    before = len(diffs)
    cmp(dfa[k], dfb[k], k)
    if len(diffs) > before:
        tree_diffs.append((k, len(diffs) - before))

print(f"== noise_settings/overworld: {nether_start} diffs")
for d in diffs[:nether_start]:
    print("  ", d)
print(f"== noise_settings/nether: {len(diffs) - nether_start} diffs (first 10)")
for d in diffs[nether_start:nether_start+10]:
    print("  ", d)
print(f"== density_function/overworld: A={len(dfa)} B={len(dfb)}")
print("  only in 1.20.1:", only_a)
print("  only in 1.21.6:", only_b)
print("  changed files:", len(tree_diffs))
for k, n in tree_diffs:
    print(f"    {k}: {n} leaf diffs")
