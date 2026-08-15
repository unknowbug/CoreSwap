import difflib, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
m3 = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal3.comp', encoding='utf-8').read()
m18 = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal18.comp', encoding='utf-8').read()
def get_fn(src, name):
    pat = 'float ' + name + '('
    i = src.index(pat)
    j = src.index('{', i)
    depth = 0; k = j
    while True:
        if src[k] == '{': depth += 1
        elif src[k] == '}':
            depth -= 1
            if depth == 0: return src[i:k+1]
        k += 1
a = get_fn(m3, 'eval_df_base')
b = get_fn(m18, 'eval_df')
print('m3.eval_df_base vs m18.eval_df diff:')
for line in list(difflib.unified_diff(a.split('\n'), b.split('\n'), lineterm=''))[:30]:
    if line.startswith(('+', '-')) and not line.startswith(('+++', '---')):
        print(' ', line[:130])
