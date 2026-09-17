# -*- coding: utf-8 -*-
"""C-6 性能 A/B 采集台（260917-04）。判据 = criteria-260917-04.md §3（同批定稿 #112）。
改自 cp1-260916-01/run_cp1.py：OUTDIR 指本块目录 + dll sha 硬门 + 双臂域路径计数门。

用法: python run_c6_ab.py <tag> <legacy|domain>
日志: .investigations/c1-c6-260917-04/cmd-output/<tag>.log ；VOID 非零退出（#160）。
"""
import io, os, re, shutil, subprocess, sys, time

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

SEED = "8576294172403134396"
RUN = r"E:\PYTHON\CoreSwap\runtime\1.20.1\java\run"
PROPS = os.path.join(RUN, "server.properties")
WORLD = os.path.join(RUN, "world")
OUTDIR = r"E:\PYTHON\CoreSwap\.investigations\c1-c6-260917-04\cmd-output"
RCONPY = r"E:\PYTHON\CoreSwap\.tmp\g3-260905-03\rcon.py"
JAVA_DIR = r"E:\PYTHON\CoreSwap\versions\1.20.1\java"
GRADLE = r"D:\gradle\gradle-8.13\bin\gradle.bat"
DLL_SHA8 = "6f7fa3ae"

tag = sys.argv[1]
mode = sys.argv[2] if len(sys.argv) > 2 else "legacy"
assert mode in ("legacy", "domain"), mode
props = ["-PlightRust=1", "-Pformprobe=1", "-Pformdrive=1", "-PlightTiming=1"]
if mode == "domain":
    props.append("-Pdomainbatch=1")
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
print("[OK] level-seed set to", SEED)

if os.path.exists(WORLD):
    shutil.rmtree(WORLD)
    print("[OK] world deleted")

env = dict(os.environ)
env["GRADLE_USER_HOME"] = r"E:\PYTHON\CoreSwap\.gradle-home"
cmd = [GRADLE, ":runServer"] + props
print("[RUN]", " ".join(cmd))
lf = io.open(logfile, "w", encoding="utf-8")
p = subprocess.Popen(cmd, cwd=JAVA_DIR, stdout=lf, stderr=subprocess.STDOUT,
                     stdin=subprocess.DEVNULL, env=env)


def tail(read_from):
    lf.flush()
    with io.open(logfile, "r", encoding="utf-8", errors="replace") as f:
        f.seek(read_from)
        return f.read()


pos = 0
done_seen = False
move5_seen = False
deadline = time.time() + 1500
try:
    while time.time() < deadline:
        time.sleep(3)
        if p.poll() is not None:
            print("[FAIL] server exited early rc=", p.returncode)
            break
        chunk = tail(pos)
        pos += len(chunk.encode("utf-8", "replace"))
        if chunk:
            sys.stdout.write(chunk)
            sys.stdout.flush()
        if not done_seen and re.search(r"Done \(", chunk):
            done_seen = True
            print("\n[OK] Done seen at", time.strftime("%H:%M:%S"))
        if done_seen and re.search(r"\[FP-DRV\] ev=move seq=5 ", chunk):
            move5_seen = True
        if move5_seen:
            time.sleep(26)
            print("\n[OK] driver complete, issuing stop")
            break
finally:
    if move5_seen or p.poll() is None:
        r = subprocess.run([sys.executable, RCONPY, "stop"], capture_output=True, text=True, timeout=30)
        print("[OK] rcon stop:", (r.stdout or "").strip()[:200], (r.stderr or "").strip()[:200])
    try:
        p.wait(timeout=120)
    except subprocess.TimeoutExpired:
        print("[WARN] kill java by PID (#153)")
        subprocess.run(["taskkill", "/F", "/PID", str(p.pid)], capture_output=True)
        p.wait(timeout=60)
    lf.close()

with io.open(logfile, "r", encoding="utf-8", errors="replace") as f:
    whole = f.read()
checks = {
    "done": whole.count("Done ("),
    "drv_move5": len(re.findall(r"\[FP-DRV\] ev=move seq=5 ", whole)),
    "lightInit_ok": whole.count("lightInit ok"),
    "fallback": whole.count("fallback vanilla"),
    "domain_hook": whole.count("[LIGHT-DOMAIN] hook armed"),
    "domain_task": len(re.findall(r"\[LIGHT-DOMAIN\] task ", whole)),
    "fp_on": whole.count("[FP-ON]"),
    "lighttiming_lines": len(re.findall(r"lightTiming", whole)),
    "dll_sha8": (re.findall(r"dll=.*sha256=(\w{8})", whole) or ["(none)"])[0].lower(),
}
print("[SELFCERT]", checks)

fail = []
if checks["done"] < 1 or checks["drv_move5"] < 1 or checks["lightInit_ok"] < 1:
    fail.append("carrier")
if checks["dll_sha8"] != DLL_SHA8:
    fail.append("dll-sha=" + checks["dll_sha8"])
if checks["fallback"] > 0:
    fail.append("fallback-nonzero(new channel)")
if checks["fp_on"] < 1:
    fail.append("formprobe-not-on")
if mode == "domain":
    if checks["domain_hook"] < 1 or checks["domain_task"] < 1:
        fail.append("domain self-proof")
else:
    if checks["domain_hook"] > 0 or checks["domain_task"] > 0:
        fail.append("legacy arm saw domain path")
if fail:
    print("[VOID] self-cert failed:", fail, "-> arm VOID per criteria #118/#160")
    sys.exit(2)
print("[OK] arm", tag, "complete")
