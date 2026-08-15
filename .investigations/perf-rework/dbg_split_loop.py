import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.setrecursionlimit(3000)
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

spec = importlib.util.spec_from_file_location("m", r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect()
g.gen_df(fd)

# 带深度限制追踪的 _gen_split_lines 包装
orig = g._gen_split_lines
depth = 0
seen = {}
def traced(df, cx, cy, cz, d=0):
    global depth
    if d > 60:
        # 打印路径
        tag = df.get("type", "") if isinstance(df, dict) else str(df)[:60]
        print(f"[DEPTH>60] type={tag} cx={cx}")
        raise RuntimeError("depth exceeded")
    # 检测明显环：同一 df id 在深路径重复
    if isinstance(df, dict):
        did = id(df)
        if did in seen and seen[did] == d:
            print(f"[CYCLE?] id={did} type={df.get('type','')} at depth {d}")
        seen[did] = d
    return orig(df, cx, cy, cz)
g._gen_split_lines = lambda df, cx, cy, cz: traced(df, cx, cy, cz)

try:
    lines = g._gen_split_lines(fd, "x", "y", "z")
    print(f"OK total lines={len(lines)}")
except RuntimeError as e:
    print(f"ABORT: {e}")
