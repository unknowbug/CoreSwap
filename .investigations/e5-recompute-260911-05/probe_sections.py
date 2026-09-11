import sys, os, importlib.util
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
spec = importlib.util.spec_from_file_location(
    "daf", r"E:\PYTHON\CoreSwap\.investigations\e5-recompute-260911-05\diff_arms_fixed.py")
src = open(r"E:\PYTHON\CoreSwap\.investigations\e5-recompute-260911-05\diff_arms_fixed.py", encoding="utf-8").read()
src = src.split("van=load_world")[0]
g = {}
exec(compile(src, "daf", "exec"), g)

for d in [r"E:\PYTHON\CoreSwap\.investigations\perf-closeout-260910-05\cmd-output\region-ns-r1",
          r"E:\PYTHON\CoreSwap\.investigations\perf-closeout-260910-05\cmd-output\region-ea-r1",
          r"E:\PYTHON\CoreSwap\.investigations\perf-closeout-260910-05\cmd-output\region-r3-r2"]:
    print("===", os.path.basename(d))
    fn = sorted(f for f in os.listdir(d) if f.endswith(".mca"))[0]
    n = 0
    for nbt in g["iter_chunks"](os.path.join(d, fn)):
        data = nbt.get("") if isinstance(nbt, dict) and "" in nbt else nbt
        if not isinstance(data, dict):
            continue
        ys = sorted(s.get("Y") for s in data.get("sections", []) if isinstance(s, dict))
        print(f"  xPos={data.get('xPos')} zPos={data.get('zPos')} nsec={len(ys)} Ymin={ys[0] if ys else None} Ymax={ys[-1] if ys else None}")
        n += 1
        if n >= 3:
            break
