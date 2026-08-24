#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Reconcile production SplineDF node/table size with WG_SPLINESTATS (537 nodes/17KB).

buildSplineNode semantics:
  - each spline {coordinate,points} object => +1 node (n = len(points) > 0)
  - each SCALAR value point          => addLeaf => +1 node (n == 0, fixedValue)
  - each nested {coordinate,points} value => recursive subnode(s)
  points registry: locations/derivatives/subIdx appended len(points) per spline node.

So: total_nodes = (#spline-object nodes) + (#scalar-leaf values)
    sum_points  = sum over spline-object nodes of len(points)
    tableBytes  = nodes*sizeof(Node) + sum_points*(3*4)   (float+float+int)
    sizeof(Node)= 5 ints = 20 bytes (locFn,locBegin,subBegin,n,fixedValue)
"""
import json, os, sys, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

BASE = r"E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld"
FILES = ["factor.json", "jaggedness.json", "offset.json"]
SIZEOF_NODE = 20  # locFn(4)+locBegin(4)+subBegin(4)+n(4)+fixedValue(4)

def load(n):
    with open(os.path.join(BASE, n), encoding="utf-8") as f:
        return json.load(f)

def find_spline(obj):
    if isinstance(obj, dict):
        if obj.get("type") == "minecraft:spline":
            return obj["spline"]
        for v in obj.values():
            r = find_spline(v)
            if r is not None: return r
    elif isinstance(obj, list):
        for v in obj:
            r = find_spline(v)
            if r is not None: return r
    return None

def count(obj, acc):
    # acc: {"spline_nodes":int,"scalar_leaves":int,"points":int,"maxDepth":int,"depthDist":Counter,"coordSpline":int}
    coord = obj.get("coordinate")
    points = obj.get("points", [])
    if isinstance(coord, dict) and coord.get("type") == "minecraft:spline":
        acc["coordSpline"] += 1   # cross-instance via coordinate!
    acc["spline_nodes"] += 1
    acc["points"] += len(points)
    for p in points:
        v = p.get("value")
        if isinstance(v, dict) and "points" in v:
            count(v, acc)
        else:
            acc["scalar_leaves"] += 1

def depth_and_dist(obj, d, acc):
    acc["depthDist"][d] += 1
    acc["maxDepth"] = max(acc["maxDepth"], d)
    for p in obj.get("points", []):
        v = p.get("value")
        if isinstance(v, dict) and "points" in v:
            depth_and_dist(v, d + 1, acc)

print("=" * 78)
print("Production SplineDF static reconciliation (overworld, non-amplified)")
print("=" * 78)
tot_nodes, tot_points = 0, 0
for fn in FILES:
    sp = find_spline(load(fn))
    acc = {"spline_nodes":0,"scalar_leaves":0,"points":0,"coordSpline":0,"maxDepth":0,
           "depthDist":collections.Counter()}
    count(sp, acc)
    depth_and_dist(sp, 0, acc)
    total_nodes = acc["spline_nodes"] + acc["scalar_leaves"]
    bytes_ = total_nodes*SIZEOF_NODE + acc["points"]*(3*4)
    print(f"\n--- {fn} ---")
    print(f"  spline-object nodes (n>0): {acc['spline_nodes']}  scalar-leaf nodes (n=0): {acc['scalar_leaves']}")
    print(f"  TOTAL nodes: {total_nodes}  sum_points: {acc['points']}")
    print(f"  max recursion depth (root=0): {acc['maxDepth']}  (levels={acc['maxDepth']+1})")
    print(f"  depth distribution: {dict(sorted(acc['depthDist'].items()))}")
    print(f"  tableBytes(~): {bytes_} B")
    print(f"  coordinate=cross-instance-spline? {acc['coordSpline']} (0 => no cross-instance nesting)")
    tot_nodes += total_nodes; tot_points += acc["points"]

print("\n" + "=" * 78)
print(f"TOTAL: {len(FILES)} spline files => nodes={tot_nodes}, sum_points={tot_points}")
print(f"  tableBytes(~) = {tot_nodes*SIZEOF_NODE + tot_points*12} B")
print("  vs WG_SPLINESTATS quoted: 537 nodes / 17KB(17112B) across '6' instances")
print("  => statically we see 3 instance files; doubling gives 2x nodes but depth")
print("     stays shallow (3-4 edges). Depth-vs-node independent of instance count.")
print("=" * 78)
