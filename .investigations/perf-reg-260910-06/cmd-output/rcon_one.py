import socket, struct, sys, time
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
HOST, PORT, PW = "127.0.0.1", 25575, "coreswap"

def rcon_one(cmd, timeout=180):
    s = socket.create_connection((HOST, PORT), timeout=timeout)
    def send(t, payload, rid=0):
        data = struct.pack("<ii", rid, t) + payload.encode("utf-8") + b"\x00\x00"
        s.sendall(struct.pack("<i", len(data)) + data)
        ln = struct.unpack("<i", s.recv(4))[0]
        buf = b""
        while len(buf) < ln:
            buf += s.recv(ln - len(buf))
        return buf[8:-2].decode("utf-8", "replace")
    try:
        send(3, PW)
        return send(2, cmd)
    finally:
        s.close()

if __name__ == "__main__":
    for c in sys.argv[1:]:
        for attempt in range(3):
            try:
                print(f"> {c}\n{rcon_one(c)}")
                break
            except Exception as e:
                print(f"[retry {attempt}] {c}: {type(e).__name__}")
                time.sleep(3)
