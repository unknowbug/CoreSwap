import re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
lines = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.spvasm', encoding='utf-8').read().split('\n')

# 找函数边界（OpFunction 到 OpFunctionEnd），统计每个函数内 val[158] 变量
funcs = []
cur = None
name_re = re.compile(r'OpName %(\S+)\s+"([^"]+)"')
for i, ln in enumerate(lines):
    s = ln.strip()
    if '= OpFunction %' in s and 'OpFunctionParameter' not in s:
        m = re.search(r'%(\S+) = OpFunction', s)
        cur = {'name': f'fn_{m.group(1)}', 'start': i, 'val158': 0}
        funcs.append(cur)
    elif cur and 'OpVariable %_ptr_Function__arr_float_uint_158 Function' in s:
        cur['val158'] += 1
    elif cur and 'OpFunctionEnd' in s:
        cur = None

for f in funcs:
    if f['val158'] > 0:
        print(f"  {f['name']}: val[158] x{f['val158']}")
if not any(f['val158'] > 0 for f in funcs):
    print('无函数有 val[158] 变量？')
