# -*- coding: utf-8 -*-
"""K2 噪声锚探针（260917-03）：domain 臂同配置 run-to-run boot，与上一 snap 对比。
用法: python run_k2.py <tag> <prev_after_json> [-PlightRust -Pdomainbatch=1]
不删 world（复用上一 boot 后状态）；Done+60s → stop → snap。基于 cp1-260917-01/run_g3_run3.py，
OUTDIR 改为本课题目录（#144/#146：新标签不复用旧日志路径）。
"""
import io, json, os, re, shutil, subprocess, sys, time

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

SEED = "8576294172403134396"
RUN = r"E:\PYTHON\CoreSwap\runtime\1.20.1\java\run"
PROPS = os.path.join(RUN, "server.properties")
REGION = os.path.join(RUN, "world", "region")
OUTDIR = r"E:\PYTHON\CoreSwap\.investigations\k2-noise-anchor-260917-03\cmd-output"
G3DIR = r"E:\PYTHON\CoreSwap\.tmp\g3-260905-03"
JAVA_DIR = r"E:\PYTHON\CoreSwap\versions\1.20.1\java"
GRADLE = r"D:\gradle\gradle-8.13\bin\gradle.bat"

tag = sys.argv[1]
prev_after = sys.argv[2]
settle_s = 60
props = sys.argv[3:]
logfile = os.path.join(OUTDIR, tag + ".log")
if os.path.exists(logfile):
    print("[FAIL] log exists (#144/#146):", logfile)
    sys.exit(3)

bak = PROPS + ".bak-formprobe"
if not os.path.exists(bak):
    shutil.copyfile(PROPS, bak)
with io.open(bak, "r", encoding="utf-8") as f:
    txt = f.read()
txt = re.sub(r"(?m)^level-seed=.*$", "level-seed=" + SEED, txt)
with io.open(PROPS, "w", encoding="utf-8") as f:
    f.write(txt)
print("[OK] seed set | tag", tag, "| same-world rerun")

env = dict(os.environ)
env["GRADLE_USER_HOME"] = r"E:\PYTHON\CoreSwap\.gradle-home"
# DSH 沙箱把 TEMP 重定向到宿主 Temp（dsh-jks3Sq），JVM 内 extractWorldgenDir/JNA 建目录被拒
# → lightInit threw → 整臂 vanilla fallback（K2-D4 VOID 实证）。固化 tmpdir 到工作区。
JT = r"E:\PYTHON\CoreSwap\.tmp\k2-260917-03\jtmp"
env["TEMP"] = JT
env["TMP"] = JT
env["JAVA_TOOL_OPTIONS"] = "-Djava.io.tmpdir=" + JT
# 防 daemon 复用吞客户端 env（#32 家族）：先停旧 daemon
subprocess.run([GRADLE, "--stop"], capture_output=True, text=True, env=env, timeout=120)
cmd = [GRADLE, ":runServer"] + props
print("[RUN]", " ".join(cmd))
lf = io.open(logfile, "w", encoding="utf-8")
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
        print("[OK] Done, settling", settle_s, "s")
        time.sleep(settle_s)
        break
r = subprocess.run([sys.executable, os.path.join(G3DIR, "rcon.py"), "stop"],
                   capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=30)
print("[OK] stop issued")
t0 = time.time()
while time.time() - t0 < 180:
    if p.poll() is not None:
        break
    time.sleep(2)
try:
    p.wait(timeout=30)
except subprocess.TimeoutExpired:
    subprocess.run(["taskkill", "/F", "/PID", str(p.pid)], capture_output=True)
    p.wait(timeout=60)
lf.close()

snap = os.path.join(OUTDIR, tag + "_light_after.json")
subprocess.run([sys.executable, os.path.join(G3DIR, "snap_light.py"), REGION, snap], check=True)

a = json.load(open(prev_after))
b = json.load(open(snap))
changed = sorted(k for k in (set(a) & set(b)) if a[k] != b[k])
n = len(set(a) & set(b))
only_a = len(set(a) - set(b))
only_b = len(set(b) - set(a))
print(f"[{tag}] changed={len(changed)}/{n} = {len(changed)/max(1,n)*100:.4f}%  onlyPrev={only_a} onlyNow={only_b}")
with open(os.path.join(OUTDIR, tag + "_changed_set.json"), "w", encoding="utf-8") as f:
    json.dump({"changed": changed, "prev": os.path.basename(prev_after)}, f)

with io.open(logfile, "r", encoding="utf-8", errors="replace") as f:
    whole = f.read()
sc = {
    "lightInit_ok": whole.count("lightInit ok"),
    "fallback": whole.count("fallback vanilla"),
    "done": whole.count("Done ("),
    "domain_hook": whole.count("[LIGHT-DOMAIN] hook armed"),
    "domain_sealed_min": min((int(m) for m in re.findall(r"sealed=(\d+)", whole)), default=0),
    "dll_sha": (re.findall(r"dll=.*sha256=(\w{8})", whole) or ["(none)"])[0],
}
print("[SELFCERT]", json.dumps(sc))
gate_ok = sc["lightInit_ok"] >= 1 and sc["done"] >= 1 and sc["domain_hook"] >= 1 and sc["dll_sha"] == "6f7fa3ae"
print("[GATE]", "PASS" if gate_ok else "VOID")
if not gate_ok:
    sys.exit(4)  # E3 教训：VOID 必须断链，下一 boot 不得以 VOID 臂产物为 prev 启动
