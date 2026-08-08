# -*- coding: utf-8 -*-
# 对比 C++ [COMP] 分量 dump 与 Java DensityProbe comps.txt（同 (name,y) 对齐，输出 |diff|>1e-5 差异）
# 用法: python compare_comps.py <cpp_dump> <java_comps>
import sys, re, collections

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

def parse_cpp(path):
    # [COMP] depth -64 0.876250
    pat = re.compile(r"\[COMP\] (\S+) (-?\d+) ([-+0-9.eE]+)")
    name_map = {"barrier": "barrierNoise", "fluid_level_floodedness": "fluidLevelFloodednessNoise",
                "fluid_level_spread": "fluidLevelSpreadNoise", "lava": "lavaNoise",
                "vein_gap": "veinGap", "vein_ridged": "veinRidged", "vein_toggle": "veinToggle"}
    out = {}
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            m = pat.search(line)
            if m:
                nm = name_map.get(m.group(1), m.group(1))
                out[(nm, int(m.group(2)))] = float(m.group(3))
    return out

def parse_java(path):
    # depth -64 0.876250
    out = {}
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            parts = line.split()
            if len(parts) == 3:
                try:
                    out[(parts[0], int(parts[1]))] = float(parts[2])
                except ValueError:
                    pass
    return out

def main():
    cpp = parse_cpp(sys.argv[1])
    java = parse_java(sys.argv[2])
    keys = sorted(set(cpp) | set(java))
    diffs = []
    for k in keys:
        if k in cpp and k in java:
            d = cpp[k] - java[k]
            if abs(d) > 1e-5:
                diffs.append((k[0], k[1], cpp[k], java[k], d))
        elif k in cpp:
            diffs.append((k[0], k[1], cpp[k], None, "cpp-only"))
        else:
            diffs.append((k[0], k[1], None, java[k], "java-only"))
    print(f"[OK] cpp={len(cpp)} java={len(java)} common={len(set(cpp)&set(java))} diff>1e-5={len(diffs)}")
    for name, y, cv, jv, d in diffs[:80]:
        print(f"[DIFF] {name} y={y} cpp={cv} java={jv} diff={d}")

if __name__ == "__main__":
    main()
