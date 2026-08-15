import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

# 递归找 jagged 节点及其上下文路径
def walk(df, path, depth=0):
    if isinstance(df, str):
        if 'jagged' in df:
            print(f"[jagged-ref] path={path} ref={df}")
        return
    if isinstance(df, (int, float)):
        return
    if isinstance(df, dict):
        t = df.get("type", "")
        has_jagged = False
        for k, v in df.items():
            if isinstance(v, str) and 'jagged' in v:
                print(f"[jagged-str] path={path} field={k} val={v}")
                has_jagged = True
        if 'points' in df and 'coordinate' in df:
            # spline
            walk(df.get('coordinate'), path + '.coord', depth+1)
            for i, p in enumerate(df.get('points', [])):
                walk(p.get('value'), path + f'.points[{i}].value', depth+1)
            return
        for k, v in df.items():
            if k in ('coordinate', 'points', 'type'):
                continue
            if isinstance(v, (dict, list, str, int, float)):
                walk(v, path + '.' + k, depth+1)
    elif isinstance(df, list):
        for i, v in enumerate(df):
            walk(v, path + f'[{i}]', depth+1)

walk(fd, 'final_density')
print("---done---")
