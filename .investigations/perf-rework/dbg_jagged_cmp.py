import json, sys, importlib.util, os

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

def load_module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m

for label, path in [("HEAD", r'E:\PYTHON\CoreSwap\_head_dfc.py'),
                    ("CURRENT", r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')]:
    try:
        m = load_module(path, f"m_{label}")
        g = m.DfcGen(dfdir, ndir)
        g.gen(fd)
        keys = sorted(g.normal_chain_index.keys())
        jagged = [k for k in keys if 'jagged' in k]
        print(f"[{label}] noise_instances={len(g.noise_instances)} normal_chain_index={len(keys)}")
        print(f"[{label}] jagged keys: {jagged}")
        # 也检查 noise_index 里的 jagged
        jn = [k for k in g.noise_index.keys() if 'jagged' in k]
        print(f"[{label}] noise_index jagged: {jn}")
    except Exception as e:
        import traceback
        print(f"[{label}] ERROR: {type(e).__name__}: {e}")
        traceback.print_exc()
    print()
