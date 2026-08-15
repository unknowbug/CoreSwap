import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
f = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld\factor.json', encoding='utf-8'))
found = []
def walk(n, path):
    if isinstance(n, dict):
        if 'points' in n and 'coordinate' in n:
            for i, p in enumerate(n['points']):
                v = p.get('value')
                if isinstance(v, (int, float)) and (abs(v - 6.3) < 0.01 or abs(v - 6.25) < 0.01):
                    found.append((path, i, v))
        for k, v in n.items():
            walk(v, path + '.' + k)
    elif isinstance(n, list):
        for i, v in enumerate(n):
            walk(v, path + '[' + str(i) + ']')
walk(f, 'factor')
print('value 6.3/6.25 的 points:', found[:5])

def show_spline(s, d=0):
    coord = s.get("coordinate", "?")
    pts = s.get("points", [])
    print('  ' * d + 'spline coord=' + str(coord) + ' points=' + str(len(pts)))
    for i, p in enumerate(pts[:4]):
        v = p.get('value')
        if isinstance(v, dict) and 'points' in v:
            show_spline(v, d + 1)
        else:
            print('  ' * (d + 1) + 'point[' + str(i) + '] loc=' + str(p.get('location')) + ' der=' + str(p.get('derivative')) + ' val=' + str(v))

def find_splines(n, d=0):
    if isinstance(n, dict):
        if 'points' in n and 'coordinate' in n:
            show_spline(n, d)
        for v in n.values():
            find_splines(v, d + 1)
    elif isinstance(n, list):
        for v in n:
            find_splines(v, d + 1)
find_splines(f)
