# 扫描 final_density 树所有 weird_scaled_sampler 节点
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings['noise_router']['final_density']
resolved = set()
def resolve_ref(ref):
    if ref in resolved: return None
    resolved.add(ref)
    name = ref.replace('minecraft:overworld/', '')
    try:
        return json.load(open(dfdir + r'\overworld\%s.json' % name, encoding='utf-8'))
    except FileNotFoundError:
        return None
def scan(x, path='fd'):
    if isinstance(x, dict):
        if x.get('type') == 'minecraft:weird_scaled_sampler':
            inp = x.get('input')
            def desc(v):
                if isinstance(v, str): return 'REF:' + v
                if isinstance(v, dict):
                    t = v.get('type', '?')
                    if t == 'minecraft:cache_once': return 'cache_once(' + desc(v.get('argument')) + ')'
                    if t == 'minecraft:noise': return 'noise:' + str(v.get('noise')) + ' xz=%s y=%s' % (v.get('xz_scale'), v.get('y_scale'))
                    if t == 'minecraft:interpolated': return 'interpolated'
                    return t
                return str(v)
            print('WS at %s: noise=%s rarity=%s input=%s' % (path, x.get('noise'), x.get('rarity_value_mapper'), desc(inp)))
            # 展开输入子树找噪声
            scan(inp, path + '.input')
            return
        for k, v in x.items():
            if k == 'type': continue
            if isinstance(v, str) and v.startswith('minecraft:'):
                sub = resolve_ref(v)
                if sub is not None:
                    scan(sub, path + '.<%s>' % v)
                continue
            scan(v, path + '.' + k)
    elif isinstance(x, list):
        for i, v in enumerate(x):
            scan(v, path + '[%d]' % i)
scan(fd)
print('resolved refs:', len(resolved))
