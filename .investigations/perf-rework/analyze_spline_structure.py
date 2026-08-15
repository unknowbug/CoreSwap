"""Analyze final_density spline structure for SSBO data-driven design (B1a).
Resolves registry references like dfc_gen.resolve_ref. Outputs coordinate kinds,
value tri-state distribution, nesting depth, locations sizes, and value kind
breakdown per spline (const / nested-spline / other-DF).
"""
import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'

registry = {}
for root, _, files in os.walk(DFDIR):
    for fn in files:
        if fn.endswith('.json'):
            rel = os.path.relpath(os.path.join(root, fn), DFDIR)[:-5]
            with open(os.path.join(root, fn), encoding='utf-8') as f:
                registry[rel] = json.load(f)

settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]

df_cache = {}
def resolve_ref(ref):
    if ref in ("minecraft:y", "minecraft:zero", "minecraft:shift_x", "minecraft:shift_z"):
        return {"type": ref}
    if ref in df_cache:
        return df_cache[ref]
    rel = ref.replace("minecraft:", "")
    fpath = os.path.join(DFDIR, rel + ".json")
    if os.path.exists(fpath):
        with open(fpath, encoding='utf-8') as f:
            df = json.load(f)
        df_cache[ref] = df
        return df
    raise ValueError(f"cannot resolve {ref}")

spline_count = 0
coord_kinds = {}
value_kinds = {'const': 0, 'nested_spline': 0, 'other_df': 0}
depth_stats = []
node_locs = []
nested_parent_depths = []
seen_splines = set()
other_df_types = {}

def classify_value(v):
    if isinstance(v, dict):
        if "points" in v and "coordinate" in v and "type" not in v:
            return 'nested_spline'
        t = v.get("type", "")
        if t == "minecraft:spline" and "spline" in v:
            return 'nested_spline'
        return 'other_df'
    return 'const'

def walk_spline(spl, depth):
    """spl = the actual {coordinate, points} dict."""
    global spline_count
    key = json.dumps(spl, sort_keys=True)
    if key in seen_splines:
        return
    seen_splines.add(key)
    spline_count += 1
    coord = spl["coordinate"]
    coord_kinds[json.dumps(coord, sort_keys=True)] = coord_kinds.get(json.dumps(coord, sort_keys=True), 0) + 1
    points = spl["points"]
    node_locs.append(len(points))
    depth_stats.append(depth)
    for p in points:
        v = p["value"]
        k = classify_value(v)
        value_kinds[k] += 1
        if k == 'nested_spline':
            nested_parent_depths.append(depth)
            inner = v if ("points" in v and "coordinate" in v) else v["spline"]
            walk_spline(inner, depth + 1)
        elif k == 'other_df':
            t = v.get("type", "<none>")
            other_df_types[t] = other_df_types.get(t, 0) + 1

def walk_df(df, depth=0, _seen=None):
    if _seen is None:
        _seen = set()
    if isinstance(df, str):
        if df.startswith("minecraft:"):
            ref = df
            if ref not in _seen:
                _seen.add(ref)
                walk_df(resolve_ref(ref), depth, _seen)
        return
    if not isinstance(df, dict):
        return
    t = df.get("type", "")
    if "points" in df and "coordinate" in df and t == "":
        walk_spline(df, depth)
        return
    if t == "minecraft:spline" and "spline" in df:
        walk_spline(df["spline"], depth)
        return
    for k, v in df.items():
        if isinstance(v, dict):
            walk_df(v, depth, _seen)
        elif isinstance(v, list):
            for it in v:
                if isinstance(it, (dict, str)):
                    walk_df(it, depth, _seen)

walk_df(fd)

print(f"spline nodes total: {spline_count}")
print(f"coord kinds: {len(coord_kinds)}")
print("top coord kinds by count:")
for ck, cnt in sorted(coord_kinds.items(), key=lambda x: -x[1])[:8]:
    print(f"  x{cnt} {ck[:130]}")
print(f"value kinds: const={value_kinds['const']} nested_spline={value_kinds['nested_spline']} other_df={value_kinds['other_df']}")
print(f"node locs sizes: min={min(node_locs)} max={max(node_locs)} avg={sum(node_locs)/len(node_locs):.1f}")
print(f"nesting depth (spline level): min={min(depth_stats)} max={max(depth_stats)}")
if nested_parent_depths:
    print(f"nested spline values: {len(nested_parent_depths)} (parent depth max={max(nested_parent_depths)})")
print(f"other-DF value types: {other_df_types}")
