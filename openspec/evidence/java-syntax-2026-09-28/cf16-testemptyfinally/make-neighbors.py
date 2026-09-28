#!/usr/bin/env python3
"""从固定 CF-16 class 冻结三份可通过 verifier 的近邻。"""
from hashlib import sha256
from pathlib import Path

ROOT = Path(__file__).resolve().parent
original = (ROOT / "original" / "TestEmptyFinally$TestCls.class").read_bytes()
assert sha256(original).hexdigest() == "dcf5de9a4037ddd2169f103e426be38fac60dba8fb2d5cb38dc6ffe3c648018a"
code = bytes.fromhex("2b b6 00 07 a7 00 0a 4d a7 00 06 4e 2d bf b1")
rows = bytes.fromhex("00 00 00 04 00 07 00 0c 00 00 00 04 00 0b 00 00")
assert original.count(code) == original.count(rows) == 1

variants = {
    # handler 改为抛出新的 NullPointerException，不再是接住的原对象。
    "changed-throwable": original.replace(code, code[:12] + b"\x01" + code[13:]),
    # catch-all 排在具名 IOException 行之前。
    "rows-swapped": original.replace(rows, rows[8:] + rows[:8]),
    # pop; return 给 catch-all handler 增加正常出口。
    "handler-normal-exit": original.replace(code, code[:13] + b"\x57" + code[14:]),
}
out = ROOT / "neighbors"
out.mkdir(exist_ok=True)
for name, content in variants.items():
    (out / f"{name}.class").write_bytes(content)
    print(f"{sha256(content).hexdigest()}  neighbors/{name}.class")
