#!/usr/bin/env python3
"""Freeze the two super-argument probes: the shapes javac never writes.

`javac` hands the *parameter* to the superclass constructor and hoists a call out of the
constructor entirely, so the two shapes this fixture freezes have no Java source: they are
hand-made bytecode, written here and frozen as class files.

* `read-arg/` — the constructor hands the **field the group moves** to the superclass
  constructor:

      aload_0; aload_1; putfield ReadArg$1.val$s;
      aload_0; aload_0; getfield ReadArg$1.val$s;
      invokespecial ReadArg$Base.<init>(Ljava/lang/String;)V; return

  The shape is not verifiable: JVMS 4.10.1.9 lets `uninitializedThis` be used for a `putfield`
  of the current class and for the `invokespecial` that initializes it, and a `getfield` on it
  is refused ("Type uninitializedThis ... is not assignable to 'ReadArg$1'"). `freeze.py`
  asserts that refusal on both runtimes.

* `call-arg/` — the constructor's argument is a **call** whose body is not in the artifact:

      aload_0; aload_1; putfield CallArg$1.val$s;
      aload_0; invokestatic CallArg$Helper.compute()Ljava/lang/String;
      invokespecial CallArg$Base.<init>(Ljava/lang/String;)V; return

  This one verifies and runs; the class declares nothing but that constructor, so the only
  refusal a reorder could meet is the argument walk's.

    python3 tests/fixtures/proved-java-structure/capture-super-arg-probes/freeze.py
"""

from pathlib import Path
import hashlib
import os
import struct
import subprocess

ROOT = Path(__file__).resolve().parent
FALLBACK_JAVA8 = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java"


def u1(value: int) -> bytes:
    return struct.pack(">B", value)


def u2(value: int) -> bytes:
    return struct.pack(">H", value)


def u4(value: int) -> bytes:
    return struct.pack(">I", value)


class Pool:
    """A constant pool, entry 1 always the `Code` attribute name every member here writes."""

    def __init__(self) -> None:
        self.entries: list[bytes] = []

    def push(self, data: bytes) -> int:
        self.entries.append(data)
        return len(self.entries)

    def utf8(self, text: str) -> int:
        encoded = text.encode()
        return self.push(u1(1) + u2(len(encoded)) + encoded)

    def cls(self, name: int) -> int:
        return self.push(u1(7) + u2(name))

    def nat(self, name: int, descriptor: int) -> int:
        return self.push(u1(12) + u2(name) + u2(descriptor))

    def fieldref(self, owner: int, name_and_type: int) -> int:
        return self.push(u1(9) + u2(owner) + u2(name_and_type))

    def methodref(self, owner: int, name_and_type: int) -> int:
        return self.push(u1(10) + u2(owner) + u2(name_and_type))

    def string(self, utf8: int) -> int:
        return self.push(u1(8) + u2(utf8))


def classfile(pool: Pool, flags: int, this: int, superclass: int, fields, methods) -> bytes:
    out = b"\xca\xfe\xba\xbe" + u2(0) + u2(52)
    out += u2(len(pool.entries) + 1)
    for entry in pool.entries:
        out += entry
    out += u2(flags) + u2(this) + u2(superclass) + u2(0)
    out += u2(len(fields))
    for field_flags, field_name, field_descriptor in fields:
        out += u2(field_flags) + u2(field_name) + u2(field_descriptor) + u2(0)
    out += u2(len(methods))
    for method_flags, method_name, method_descriptor, (stack, locals_, code) in methods:
        out += u2(method_flags) + u2(method_name) + u2(method_descriptor) + u2(1) + u2(1)
        attribute = u2(stack) + u2(locals_) + u4(len(code)) + code + u2(0) + u2(0)
        out += u4(len(attribute)) + attribute
    out += u2(0)
    return out


def scaffolding(pool: Pool, host: str):
    """The base class, the object/system references and the host's constructor call."""
    object_class = pool.cls(pool.utf8("java/lang/Object"))
    init = pool.utf8("<init>")
    object_init = pool.methodref(object_class, pool.nat(init, pool.utf8("()V")))
    system = pool.cls(pool.utf8("java/lang/System"))
    out = pool.fieldref(system, pool.nat(pool.utf8("out"), pool.utf8("Ljava/io/PrintStream;")))
    stream = pool.cls(pool.utf8("java/io/PrintStream"))
    println = pool.methodref(stream, pool.nat(pool.utf8("println"), pool.utf8("(Ljava/lang/String;)V")))

    base = pool.cls(pool.utf8(f"{host}$Base"))
    base_descriptor = pool.utf8("(Ljava/lang/String;)V")
    base_init = pool.methodref(base, pool.nat(init, base_descriptor))
    base_code = bytes([0x2a, 0xb7]) + u2(object_init)
    base_code += bytes([0xb2]) + u2(out) + bytes([0x2b])
    base_code += bytes([0xb6]) + u2(println) + bytes([0xb1])
    base_class = classfile(pool, 0x0020, base, object_class, [], [
        (0x0000, init, base_descriptor, (2, 2, base_code)),
    ])
    return object_class, base, base_init, base_descriptor, base_class


def host(pool: Pool, object_class: int, name: str, child: int, child_init: int) -> bytes:
    literal = pool.string(pool.utf8("z"))
    assert literal < 256, "ldc takes a one-byte index"
    code = bytes([0xbb]) + u2(child) + bytes([0x59, 0x12, literal])
    code += bytes([0xb7]) + u2(child_init) + bytes([0xb1])
    return classfile(pool, 0x0021, pool.cls(pool.utf8(name)), object_class, [], [
        (0x0009, pool.utf8("main"), pool.utf8("([Ljava/lang/String;)V"), (3, 1, code)),
    ])


def read_arg() -> dict[str, bytes]:
    """The argument is read from the field the group moves (unverifiable bytecode)."""
    pool = Pool()
    assert pool.utf8("Code") == 1
    object_class, base, base_init, base_descriptor, base_class = scaffolding(pool, "ReadArg")
    child = pool.cls(pool.utf8("ReadArg$1"))
    val = pool.utf8("val$s")
    descriptor = pool.utf8("Ljava/lang/String;")
    field = pool.fieldref(child, pool.nat(val, descriptor))
    child_init = pool.methodref(child, pool.nat(pool.utf8("<init>"), base_descriptor))
    code = bytes([0x2a, 0x2b, 0xb5]) + u2(field)
    code += bytes([0x2a, 0x2a, 0xb4]) + u2(field)
    code += bytes([0xb7]) + u2(base_init) + bytes([0xb1])
    return {
        "ReadArg$Base.class": base_class,
        "ReadArg$1.class": classfile(pool, 0x0020, child, base, [
            (0x1010, val, descriptor),
        ], [
            (0x0000, pool.utf8("<init>"), base_descriptor, (2, 2, code)),
        ]),
        "ReadArg.class": host(pool, object_class, "ReadArg", child, child_init),
    }


def call_arg() -> dict[str, bytes]:
    """The argument is a call whose body the artifact does not hold (verifiable bytecode)."""
    pool = Pool()
    assert pool.utf8("Code") == 1
    object_class, base, base_init, base_descriptor, base_class = scaffolding(pool, "CallArg")
    helper = pool.cls(pool.utf8("CallArg$Helper"))
    compute_descriptor = pool.utf8("()Ljava/lang/String;")
    compute = pool.methodref(helper, pool.nat(pool.utf8("compute"), compute_descriptor))
    literal = pool.string(pool.utf8("c"))
    assert literal < 256
    child = pool.cls(pool.utf8("CallArg$1"))
    val = pool.utf8("val$s")
    descriptor = pool.utf8("Ljava/lang/String;")
    field = pool.fieldref(child, pool.nat(val, descriptor))
    child_init = pool.methodref(child, pool.nat(pool.utf8("<init>"), base_descriptor))
    code = bytes([0x2a, 0x2b, 0xb5]) + u2(field)
    code += bytes([0x2a, 0xb8]) + u2(compute)
    code += bytes([0xb7]) + u2(base_init) + bytes([0xb1])
    return {
        "CallArg$Base.class": base_class,
        "CallArg$Helper.class": classfile(pool, 0x0020, helper, object_class, [], [
            (0x0009, pool.utf8("compute"), compute_descriptor, (1, 0, bytes([0x12, literal, 0xb0]))),
        ]),
        "CallArg$1.class": classfile(pool, 0x0020, child, base, [
            (0x1010, val, descriptor),
        ], [
            (0x0000, pool.utf8("<init>"), base_descriptor, (2, 2, code)),
        ]),
        "CallArg.class": host(pool, object_class, "CallArg", child, child_init),
    }


def write(leg: str, classes: dict[str, bytes]) -> None:
    destination = ROOT / leg
    destination.mkdir(exist_ok=True)
    for name, data in classes.items():
        (destination / name).write_bytes(data)


def run(directory: Path, main: str, java: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [java, "-Xverify:all", "-cp", str(directory), main],
        text=True,
        capture_output=True,
    )


write("read-arg", read_arg())
write("call-arg", call_arg())

java8 = Path(os.environ.get("JARDE_JAVA8", FALLBACK_JAVA8))
runtimes = [("java", "java")]
if java8.exists():
    runtimes.append(("java8", str(java8)))
else:
    print(f"note: no JDK 8 at {java8}; the JDK 8 leg is not run here")

for label, runtime in runtimes:
    refused = run(ROOT / "read-arg", "ReadArg", runtime)
    assert refused.returncode != 0 and "VerifyError" in refused.stderr, (
        f"{label}: the field-read probe is expected to be refused by the verifier",
        refused.returncode,
        refused.stderr,
    )
    print(f"{label}: read-arg refused by the verifier ({refused.stderr.splitlines()[0]})")
    accepted = run(ROOT / "call-arg", "CallArg", runtime)
    assert accepted.returncode == 0 and accepted.stdout == "c\n", (label, accepted.stdout, accepted.stderr)
    print(f"{label}: call-arg prints {accepted.stdout!r}")

with open(ROOT / "SHA256SUMS", "w") as sums:
    for leg in ("read-arg", "call-arg"):
        for name in sorted(os.listdir(ROOT / leg)):
            digest = hashlib.sha256((ROOT / leg / name).read_bytes()).hexdigest()
            sums.write(f"{digest}  {leg}/{name}\n")
print("frozen")
