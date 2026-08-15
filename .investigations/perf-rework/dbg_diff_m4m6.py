import difflib, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
m4 = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal4.comp', encoding='utf-8').read()
m6 = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal6.comp', encoding='utf-8').read()
def get_fn(src, name):
    i = src.index('float ' + name + '(')
    depth = 0; j = src.index('{', i); k = j
    while True:
        if src[k] == '{': depth += 1
        elif src[k] == '}':
            depth -= 1
            if depth == 0: return src[i:k+1]
        k += 1
for fn in ('eval_df', 'interp_0', 'eval_df_base'):
    a, b = get_fn(m4, fn), get_fn(m6, fn)
    same = a == b
    tag = 'same' if same else 'DIFF'
    print(f'=== {fn}: {tag} ===')
    if not same:
        for line in list(difflib.unified_diff(a.split(chr(10)), b.split(chr(10)), lineterm=''))[:14]:
            if line.startswith(('+', '-')) and not line.startswith(('+++', '---')):
                print(' ', line[:110])
