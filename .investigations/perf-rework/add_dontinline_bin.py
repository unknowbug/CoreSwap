# add_dontinline_bin.py —— 给所有非 entry 函数的 OpFunction FunctionControl 设 DontInline 位（bit 1）
import struct, sys

def main(inp, outp):
    with open(inp, 'rb') as f:
        data = f.read()
    words = list(struct.unpack('<%dI' % (len(data) // 4), data))

    # 找 OpEntryPoint（opcode 15）的 entry 函数 ID
    entry_func = None
    i = 5
    while i < len(words):
        w0 = words[i]
        op = w0 & 0xFFFF
        wc = w0 >> 16
        if op == 15:
            entry_func = words[i + 2]
            break
        i += wc

    # 遍历所有 OpFunction（opcode 54），给非 entry 函数设 DontInline 位
    count = 0
    i = 5
    while i < len(words):
        w0 = words[i]
        op = w0 & 0xFFFF
        wc = w0 >> 16
        if op == 54:  # OpFunction: [wc|54, result_type, result_id, function_control, function_type]
            fid = words[i + 2]
            if fid != entry_func:
                words[i + 3] |= 0x2  # FunctionControlDontInline = bit 1
                count += 1
        i += wc

    with open(outp, 'wb') as f:
        f.write(struct.pack('<%dI' % len(words), *words))
    print(f"entry=%{entry_func} 给 {count} 个非 entry 函数设了 DontInline -> {outp}")

if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
