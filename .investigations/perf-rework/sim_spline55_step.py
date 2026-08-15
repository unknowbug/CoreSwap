# sim_spline55_step.py —— spline_eval(55) 逐步 trace
import sys, os, json, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib.util
spec = importlib.util.spec_from_file_location('sim', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
sim.splitCoord = struct.unpack('f' * 8672, open(base + r'\split_single.bin', 'rb').read())
sim.SPLIT_TOTAL = 8672

# monkey-patch spline_eval_py 加 trace
orig = sim.spline_eval_py
import inspect
src_lines = inspect.getsource(orig).split('\n')
# 太复杂，直接手动逐步：复制核心逻辑
svk, svf, svn = sim.svk, sim.svf, sim.svn
slocs, sders, snodes = sim.slocs, sim.sders, sim.snodes
nodeStack = [0]*24; stageStack = [0]*24; iStack = [0]*24; outSlot = [0]*24
v0Stack = [0.0]*24; v1Stack = [0.0]*24; coordStack = [0.0]*24
sp = 0; nodeStack[0] = 55; stageStack[0] = 0; iStack[0] = 0; outSlot[0] = -1
result = 0.0; steps = 0
while sp >= 0:
    steps += 1
    if steps > 50: print('LOOP'); break
    node = nodeStack[sp]
    nd = snodes[node]
    ct, n, lb, db, vb = nd['coordType'], nd['n'], nd['locBegin'], nd['derBegin'], nd['valBegin']
    st = stageStack[sp]
    print(f'step{steps} sp={sp} node={node} stage={st} n={n} outSlot={outSlot[sp]}', end='')
    if st == 0:
        coord = sim.spline_coord_py(ct, 0, 0, 784, 160, -408)
        coordStack[sp] = coord
        i = sim.spline_find_range(coord, lb, n)
        print(f' coord={coord:.6f} i={i}', end='')
        if i < 0:
            vk = svk[vb]
            if vk == 0:
                result = svf[vb] + sders[db] * (coord - slocs[lb])
                ps = outSlot[sp]; sp -= 1
                print(f' L-BOUND const result={result:.6f} ps={ps}')
                if ps >= 0:
                    if (ps & 1) == 0: v0Stack[ps >> 1] = result
                    else: v1Stack[ps >> 1] = result
                    stageStack[ps >> 1] = 2
                continue
            print(' L-BOUND nested -> push', end='')
            outSlot[sp] = -1; stageStack[sp] = 6; sp += 1
            nodeStack[sp] = svn[vb]; stageStack[sp] = 0; iStack[sp] = 0
            v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
            outSlot[sp] = (sp - 2) * 2
            print(f' child={svn[vb]} outSlot={outSlot[sp]}')
            continue
        if i >= n - 1:
            vk = svk[vb + n - 1]
            if vk == 0:
                result = svf[vb + n - 1] + sders[db + n - 1] * (coord - slocs[lb + n - 1])
                ps = outSlot[sp]; sp -= 1
                print(f' R-BOUND const result={result:.6f} ps={ps}')
                if ps >= 0:
                    if (ps & 1) == 0: v0Stack[ps >> 1] = result
                    else: v1Stack[ps >> 1] = result
                    stageStack[ps >> 1] = 2
                continue
            print(' R-BOUND nested -> push', end='')
            outSlot[sp] = -1; stageStack[sp] = 7; sp += 1
            nodeStack[sp] = svn[vb + n - 1]; stageStack[sp] = 0; iStack[sp] = 0
            v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
            outSlot[sp] = (sp - 2) * 2 + 1
            print(f' child={svn[vb+n-1]} outSlot={outSlot[sp]}')
            continue
        print(' MID', end='')
        iStack[sp] = i
        vk0 = svk[vb + i]
        if vk0 == 0:
            v0Stack[sp] = svf[vb + i]; stageStack[sp] = 1
            print(' v0=const')
        else:
            stageStack[sp] = 1; sp += 1
            nodeStack[sp] = svn[vb + i]; stageStack[sp] = 0; iStack[sp] = 0
            outSlot[sp] = (sp - 1) * 2
            v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
            print(f' v0=nested push {svn[vb+i]}')
        continue
    if st == 1:
        i = iStack[sp]
        vk1 = svk[vb + i + 1]
        if vk1 == 0:
            v1Stack[sp] = svf[vb + i + 1]; stageStack[sp] = 2
            print(' v1=const')
        else:
            stageStack[sp] = 2; sp += 1
            nodeStack[sp] = svn[vb + i + 1]; stageStack[sp] = 0; iStack[sp] = 0
            outSlot[sp] = (sp - 1) * 2 + 1
            v0Stack[sp] = 0.0; v1Stack[sp] = 0.0; coordStack[sp] = 0.0
            print(f' v1=nested push {svn[vb+i+1]}')
        continue
    if st == 2:
        i = iStack[sp]
        coord = coordStack[sp]; nv = v0Stack[sp]; ov = v1Stack[sp]
        span = slocs[lb + i + 1] - slocs[lb + i]
        kd = (coord - slocs[lb + i]) / span
        p = sders[db + i] * span - (ov - nv)
        q = -sders[db + i + 1] * span + (ov - nv)
        result = (nv + kd * (ov - nv)) + kd * (1.0 - kd) * (p + kd * (q - p))
        ps = outSlot[sp]; sp -= 1
        print(f' HERMITE result={result:.6f} ps={ps}')
        if ps >= 0:
            if (ps & 1) == 0: v0Stack[ps >> 1] = result
            else: v1Stack[ps >> 1] = result
            if stageStack[ps >> 1] in (6, 7):
                print(f'  -> parent stage={stageStack[ps >> 1]} (boundary, keep)')
                continue
            stageStack[ps >> 1] = 2
        continue
    if st == 6:
        pnd = snodes[node]
        c2 = coordStack[sp]
        result = v0Stack[sp] + sders[pnd['derBegin']] * (c2 - slocs[pnd['locBegin']])
        ps = outSlot[sp]; sp -= 1
        print(f' BOUND-L result={result:.6f} ps={ps}')
        if ps >= 0:
            if (ps & 1) == 0: v0Stack[ps >> 1] = result
            else: v1Stack[ps >> 1] = result
            if stageStack[ps >> 1] in (6, 7): continue
            stageStack[ps >> 1] = 2
        continue
    if st == 7:
        pnd = snodes[node]
        c2 = coordStack[sp]
        result = v1Stack[sp] + sders[pnd['derBegin'] + pnd['n'] - 1] * (c2 - slocs[pnd['locBegin'] + pnd['n'] - 1])
        ps = outSlot[sp]; sp -= 1
        print(f' BOUND-R result={result:.6f} ps={ps}')
        if ps >= 0:
            if (ps & 1) == 0: v0Stack[ps >> 1] = result
            else: v1Stack[ps >> 1] = result
            if stageStack[ps >> 1] in (6, 7): continue
            stageStack[ps >> 1] = 2
        continue
    print()
print(f'final result = {result}, steps = {steps}')
