# minimal16 = minimal6（完整版分支 interp_0）删 interp_1..4 定义 → 最小执行链（1 interp）
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal6.comp', encoding='utf-8').read()

def extract_fn(src, name):
    """返回 (函数定义文本, 结束索引)。用大括号匹配（只匹配定义，带 {）。"""
    pat = 'float ' + name + '(int sIdx, int ix, int iy, int iz) {'
    i = src.index(pat)
    j = src.index('{', i)
    depth = 0; k = j
    while True:
        if src[k] == '{': depth += 1
        elif src[k] == '}':
            depth -= 1
            if depth == 0:
                return src[i:k+1], k+1
        k += 1

# 删 interp_1..4 定义
for fn in ('interp_4', 'interp_3', 'interp_2', 'interp_1'):
    try:
        text, end = extract_fn(comp, fn)
        comp = comp.replace(text, '', 1)
        print(f'deleted {fn} ({len(text)} chars)')
    except Exception as e:
        print(f'{fn}: {e}')
# 删 interp_1..4 前向声明
for fn in ('interp_1', 'interp_2', 'interp_3', 'interp_4'):
    comp = comp.replace(f'float {fn}(int sIdx, int ix, int iy, int iz);\n', '', 1)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal16.comp', 'w', encoding='utf-8').write(comp)
print('minimal16 生成')
