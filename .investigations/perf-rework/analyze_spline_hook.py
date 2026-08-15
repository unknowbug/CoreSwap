"""B1a: collect spline structure stats by hooking dfc_gen._gen_spline (the
verified traversal). Mirrors gen_final_density.py exactly, then inspects each
spline dict as dfc_gen sees it."""
import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]

g = dfc_gen.DfcGen(DFDIR, NDIR)

seen = set()
splines = []
coord_kinds = {}
value_kinds = {'const': 0, 'nested_spline': 0, 'other_df': 0}
depths = []
locs_sizes = []
other_df_types = {}

orig_gen_spline = dfc_gen.DfcGen._gen_spline

def hooked(self, spline):
    key = json.dumps(spline, sort_keys=True)
    if key not in seen:
        seen.add(key)
        splines.append(spline)
        coord = spline["coordinate"]
        ck = json.dumps(coord, sort_keys=True)
        coord_kinds[ck] = coord_kinds.get(ck, 0) + 1
        n = len(spline["points"])
        locs_sizes.append(n)
        depth = 0
        cur = spline
        while cur is not None:
            depth += 1
            # count value kinds at this level
            for p in cur["points"]:
                v = p["value"]
                if isinstance(v, dict):
                    if "points" in v and "coordinate" in v and "type" not in v:
                        value_kinds['nested_spline'] += 1
                    elif v.get("type") == "minecraft:spline" and "spline" in v:
                        value_kinds['nested_spline'] += 1
                    else:
                        value_kinds['other_df'] += 1
                        t = v.get("type", "<none>")
                        other_df_types[t] = other_df_types.get(t, 0) + 1
                else:
                    value_kinds['const'] += 1
            # descend to first nested spline to measure depth
            nxt = None
            for p in cur["points"]:
                v = p["value"]
                if isinstance(v, dict):
                    if "points" in v and "coordinate" in v and "type" not in v:
                        nxt = v; break
                    if v.get("type") == "minecraft:spline" and "spline" in v:
                        nxt = v["spline"]; break
            cur = nxt
        depths.append(depth)
    return orig_gen_spline(self, spline)

dfc_gen.DfcGen._gen_spline = hooked

# run the same top-level gen as gen_final_density.py
expr = g.gen(fd)
g.gen_shader(fd)

print(f"spline nodes total: {len(splines)}")
print(f"coord kinds: {len(coord_kinds)}")
print("top coord kinds:")
for ck, cnt in sorted(coord_kinds.items(), key=lambda x: -x[1])[:6]:
    print(f"  x{cnt} {ck[:120]}")
print(f"value kinds: const={value_kinds['const']} nested_spline={value_kinds['nested_spline']} other_df={value_kinds['other_df']}")
print(f"locs sizes: min={min(locs_sizes)} max={max(locs_sizes)} avg={sum(locs_sizes)/len(locs_sizes):.1f}")
print(f"chain depth (spline nesting levels): min={min(depths)} max={max(depths)}")
print(f"other-DF value types: {other_df_types}")
