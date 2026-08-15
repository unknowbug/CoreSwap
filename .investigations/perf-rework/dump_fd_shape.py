"""Debug: dump final_density structure shape."""
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]

def shape(d, indent=0, path=""):
    if not isinstance(d, dict):
        return
    t = d.get("type", "<no-type>")
    print("  "*indent + f"{path or '/'} type={t} keys={list(d.keys())[:8]}")
    if "spline" in d:
        print("  "*(indent+1) + f"spline present: coordinate={json.dumps(d['spline'].get('coordinate'))[:80]} points={len(d['spline'].get('points',[]))}")
        s = d["spline"]
        for i, p in enumerate(s.get("points", [])[:3]):
            print("  "*(indent+2) + f"point[{i}] location={p.get('location')} value={json.dumps(p.get('value'))[:100]}")
    for k, v in list(d.items())[:12]:
        if isinstance(v, dict):
            shape(v, indent+1, path+"/"+k)
        elif isinstance(v, list):
            for i, it in enumerate(v[:3]):
                if isinstance(it, dict):
                    shape(it, indent+1, path+f"/{k}[{i}]")

shape(fd)
