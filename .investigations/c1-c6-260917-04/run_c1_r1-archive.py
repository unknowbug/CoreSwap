# -*- coding: utf-8 -*-
"""C-1 R1 正式判据采集台（260917-04）。判据 = .investigations/c1-c6-260917-04/criteria-260917-04.md
（同批定稿 #112）。改自 cp1-260917-01/run_g3_settle.py + run_g3_run3.py。

用法: python run_c1_r1.py <tag> <domain|legacy>
domain: -PlightRust=1 -Pdomainbatch=1 ; legacy: -PlightRust=1
协议: 删 world → run1/run2/run3 各 Done+60s → snap；判 changed(run2→run3)。
VOID/未满足均非零退出（#160）。
"""
import io, json, os, re, shutil, subprocess, sys, time

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

SEED = "8576294172403134396"
RUN = r"E:\PYTHON\CoreSwap\runtime\1.20.1\java\run"
PROPS = os.path.join(RUN, "server.properties")
WORLD = os.path.join(RUN, "world")
REGION = os.path.join(WORLD, "region")
OUTDIR = r"E:\PYTHON\CoreSwap\.investigations\c1-c6-260917-04\cmd-output"
G3DIR = r"E:\PYTHON\CoreSwap\.tmp\g3-260905-03"
JAVA_DIR = r"E:\PYTHON\CoreSwap\versions\1.20.1\java"
GRADLE = r"D:\gradle\gradle-8.13\bin\gradle.bat"
DLL_SHA8 = "6f7fa3ae"
SETTLE = 60

tag = sys.argv[1]
mode = sys.argv[2] if len(sys.argv) > 2 else "domain"
assert mode in ("domain", "legacy"), mode
props = ["-PlightRust=1"] + (["-Pdomainbatch=1"] if mode == "domain" else [])
os.makedirs(OUTDIR, exist_ok=True)
logfile = os.path.join(OUTDIR, tag + ".log")
if os.path.exists(logfile):
    print("[FAIL] log exists, pick a new tag (#144/#146):", logfile)
    sys.exit(3)

bak = PROPS + ".bak-formprobe"
if not os.path.exists(bak):
    shutil.copyfile(PROPS, bak)
with io.open(bak, "r", encoding="utf-8") as f:
    txt = f.read()
txt = re.sub(r"(?m)^level-seed=.*$", "level-seed=" + SEED, txt)
with io.open(PROPS, "w", encoding="utf-8") as f:
    f.write(txt)
print("[OK] seed set", SEED, "| mode", mode, "| tag", tag)


def boot_and_stop(boot_label):
    env = dict(os.environ)
    env["GRADLE_USER_HOME"] = r"E:\PYTHON\CoreSwap\.gradle-home"
    cmd = ["D:\\gradle\\gradle-8.13\\bin\\gradle.bat", ":runServer"] + props
    print("[RUN]", boot_label, " ".join(cmd))
    lf = io.open(logfile, "a", encoding="utf-8")
    lf.write("\n===== %s =====\n" % boot_label)
    lf.flush()
    p = subprocess.Popen(cmd, cwd=JAVA_DIR, stdout=lf, stderr=subprocess.STDOUT,
                         stdin=subprocess.DEVNULL, env=env)
    lf.flush()
    pos = os.path.getsize(logfile)
    done = False
    deadline = time.time() + 900
    while time.time() < deadline:
        time.sleep(2)
        if p.poll() is not None:
            print("[FAIL] early exit rc=", p.returncode)
            break
        lf.flush()
        with io.open(logfile, "r", encoding="utf-8", errors="replace") as f:
            f.seek(pos)
            chunk = f.read()
            pos = f.tell()
        if not done and re.search(r"Done \(", chunk):
            done = True
            print("[OK] Done, settling", SETTLE, "s")
            time.sleep(SETTLE)
            break
    r = subprocess.run([sys.executable, os.path.join(G3DIR, "rcon.py"), "stop"],
                       capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=30)
    print("[OK] stop issued", (r.stdout or r.stderr or "").strip()[:120])
    t0 = time.time()
    while time.time() - t0 < 180:
        if p.poll() is not None:
            break
        time.sleep(2)
    try:
        p.wait(timeout=30)
    except subprocess.TimeoutExpired:
        print("[WARN] gradle not exiting, kill java by PID (#153)")
        subprocess.run(["taskkill", "/F", "/PID", str(p.pid)], capture_output=True)
        p.wait(timeout=60)
    lf.close()


def boot_sections():
    with io.open(logfile, "r", encoding="utf-8", errors="replace") as f:
        whole = f.read()
    parts = re.split(r"===== (run\d) =====", whole)
    out = {}
    for i in range(1, len(parts) - 1, 2):
        out[parts[i]] = parts[i + 1]
    return out


with io.open(logfile, "w", encoding="utf-8") as f:
    f.write("")

if os.path.exists(WORLD):
    shutil.rmtree(WORLD)
    print("[OK] world deleted")

snaps = {}
for n in (1, 2, 3):
    boot_and_stop("run%d" % n)
    snap = os.path.join(OUTDIR, "%s_light_run%d.json" % (tag, n))
    subprocess.run([sys.executable, os.path.join(G3DIR, "snap_light.py"), REGION, snap], check=True)
    snaps[n] = snap

a = json.load(open(snaps[2]))
b = json.load(open(snaps[3]))
common = set(a) & set(b)
changed = sorted(k for k in common if a[k] != b[k])
print(f"[{tag}] run2->run3 common={len(common)} changed={len(changed)} = {len(changed)/max(1,len(common))*100:.4f}%")

# warm-up 参考（不判）
a1 = json.load(open(snaps[1]))
ch12 = sum(1 for k in (set(a1) & set(a)) if a1[k] != a[k])
print(f"[{tag}] run1->run2 warm-up changed={ch12} (reference only, not judged)")

# SELFCERT 逐 boot 硬门（criteria §2）
void = []
per_boot = {}
for name, seg in sorted(boot_sections().items()):
    sc = {
        "done": seg.count("Done ("),
        "lightInit_ok": seg.count("lightInit ok"),
        "domain_hook": seg.count("[LIGHT-DOMAIN] hook armed"),
        "fallback": seg.count("fallback vanilla"),
        "dll_sha8": (re.findall(r"dll=.*sha256=(\w{8})", seg) or ["(none)"])[0].lower(),
    }
    per_boot[name] = sc
    if sc["done"] < 1 or sc["lightInit_ok"] < 1:
        void.append(name + ":carrier")
    if mode == "domain" and sc["domain_hook"] < 1:
        void.append(name + ":domain-hook-missing")
    if mode == "legacy" and sc["domain_hook"] > 0:
        void.append(name + ":legacy-saw-domain")
    if sc["dll_sha8"] != DLL_SHA8:
        void.append(name + ":dll-sha=" + sc["dll_sha8"])
    if sc["fallback"] > 0:
        void.append(name + ":fallback-nonzero(new channel, case-open)")
print("[SELFCERT]", json.dumps(per_boot))

result = {"mode": mode, "common": len(common), "changed_2to3": len(changed),
          "warmup_1to2": ch12, "per_boot": per_boot, "changed_set": changed}
with open(os.path.join(OUTDIR, tag + "_result.json"), "w", encoding="utf-8") as f:
    json.dump(result, f, indent=1)

if void:
    print("[VOID] self-cert failed:", void, "-> per criteria #118/#160")
    sys.exit(2)

if mode == "domain":
    n = len(changed)
    if n <= 1:
        print("[C-1 R1] SATISFIED (changed %d <= 1)" % n)
        sys.exit(0)
    elif n <= 2:
        print("[C-1 R1] CONSERVATIVE-SATISFIED (changed %d <= 2, primary band missed)" % n)
        sys.exit(0)
    else:
        print("[C-1 R1] NOT-SATISFIED (changed %d >= 3)" % n)
        sys.exit(1)
else:
    print("[C-1 R1] legacy form-control: changed=%d (not band-judged, form reference only)" % len(changed))
    sys.exit(0)
