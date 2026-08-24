#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Static analyzer: production SplineDF intra-instance tree structure.

Measures, per production minecraft:spline instance (factor/jaggedness/offset),
the INTRA-instance spline node tree:
  - max recursion depth (root -> deepest subnode, counting value-subnode edges)
  - node count (subnode/leaf nodes; n==0 => leaf)
  - leaf count, max branching factor (n points), per-depth node distribution
  - whether each node's coordinate resolves to a *spline* (cross-instance) or not

Key: coordinate (locFn) is a STRING ref -> a separate DF (flat_cache/binary),
NOT a SplineDF. So sampleNode recursion depth == intra-tree depth only.
"""
import json, os, sys, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

BASE = r"E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld"
FILES = ["factor.json", "jaggedness.json", "offset.json"]

def load(name):
    with open(os.path.join(BASE, name), encoding="utf-8") as f:
        return json.load(f)

def find_spline(obj):
    """Return the first minecraft:spline 'spline' sub-object (coordinate,points)."""
    if isinstance(obj, dict):
        if obj.get("type") == "minecraft:spline":
            return obj["spline"]
        for v in obj.values():
            r = find_spline(v)
            if r is not None:
                return r
    elif isinstance(obj, list):
        for v in obj:
            r = find_spline(v)
            if r is not None:
                return r
    return None

def is_number(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool)

def analyze_spline(spline_obj, name):
    """Build intra-instance node tree; return structure stats."""
    nodes = []          # list of dicts: {depth, n}
    coord_types = collections.Counter()
    stats = {"name": name, "instances": 1}
    maxDepth = [0]
    maxBranch = [0]

    def walk(node_obj, depth):
        coord = node_obj.get("coordinate")
        points = node_obj.get("points", [])
        maxBranch[0] = max(maxBranch[0], len(points))
        # classify coordinate
        if isinstance(coord, dict):
            ct = coord.get("type", "?")
        elif isinstance(coord, str):
            ct = "ref:" + coord
        else:
            ct = "?" + repr(type(coord).__name__)
        coord_types[ct] += 1
        nodes.append({"depth": depth, "n": len(points), "coord": ct})
        maxDepth[0] = max(maxDepth[0], depth)
        for p in points:
            val = p.get("value")
            if isinstance(val, dict) and "points" in val:
                walk(val, depth + 1)
            # number -> leaf (no recursion)

    walk(spline_obj, 0)
    stats["maxDepth"] = maxDepth[0]
    stats["nodeCount"] = len(nodes)
    stats["leafCount"] = sum(1 for nd in nodes if nd["n"] == 0)
    # leaves here = nodes with no subnode children. A node with n points where ALL
    # values are numbers is a "terminal spline node" (n>0) whose children are leaf
    # scalars; count true scalar leaves.
    stats["maxBranch"] = maxBranch[0]
    stats["coordTypes"] = dict(coord_types)
    # depth distribution
    depthDist = collections.Counter(nd["depth"] for nd in nodes)
    stats["depthDist"] = dict(sorted(depthDist.items()))
    stats["nodes"] = nodes
    return stats

print("=" * 72)
print("Production SplineDF INTRA-INSTANCE tree (static from overworld JSON)")
print("=" * 72)
tot_nodes = 0
tot_leaf = 0
for fn in FILES:
    js = load(fn)
    spline = find_spline(js)
    if spline is None:
        print(f"[{fn}] NO minecraft:spline found")
        continue
    st = analyze_spline(spline, fn)
    print(f"\n--- {fn} (1 SplineDF instance) ---")
    print(f"  max recursion depth (root->deepest subnode): {st['maxDepth']}")
    print(f"  node count: {st['nodeCount']}   scalar-leaf count: {st['leafCount']}")
    print(f"  max n (branching points): {st['maxBranch']}")
    print(f"  depth distribution: {st['depthDist']}")
    print(f"  coordinate(locFn) type counts: {st['coordTypes']}")
    # nested coordinate spline check
    nested_spline_coord = [c for c in st['coordTypes'] if isinstance(c, str) and c.startswith('ref:')]
    # find sub-node depth list
    depths = [nd["depth"] for nd in st["nodes"]]
    print(f"  nodes at each depth: " + ", ".join(f"d{d}={depths.count(d)}" for d in sorted(set(depths))))
    tot_nodes += st["nodeCount"]
    tot_leaf += st["leafCount"]

print("\n" + "=" * 72)
print(f"TOTAL across {len(FILES)} spline-type files: nodeCount={tot_nodes}, scalarLeaf={tot_leaf}")
print("NOTE: each file = 1 SplineDF instance (buildSpline makes ONE SplineDF,")
print("      nested {coordinate,points} become INTRA-instance subnodes).")
print("NOTE: coordinates are STRING refs -> separate DF (flat_cache/binary).")
print("      NO cross-instance SplineDF nesting via coordinate in vanilla data.")
print("=" * 72)
