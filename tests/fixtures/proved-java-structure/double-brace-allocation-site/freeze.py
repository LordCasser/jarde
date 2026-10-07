#!/usr/bin/env python3
"""Freeze the double-brace allocation point's controls: one positive, three negatives.

The positive control is the source shape itself — `new ArrayList<String>() {{ add(s); }}` in a
static factory — and the three negatives are the shapes the four-criteria admission must refuse,
each differing from the control in exactly one criterion:

* `controls/single-site/` — the control: an anonymous subclass of a platform collection whose body
  is the pure instance block, allocated exactly once, with a spellable superclass. The allocation
  point presents the double-brace form (both legs).
* `negatives/methods/` — the companion declares a method (`void mark()`): its body is not a pure
  instance block, so the presentation keeps the companion's own class and its `new DBS$1(s)` call.
* `negatives/unspellable-super/` — the superclass is a nested class (`Carrier$Nested`), whose pool
  form is not a source name: the allocation point keeps the companion call.
* `negatives/multi-site/` — the companion is allocated at **two** points. No Java source states
  that shape (one anonymous class body is one allocation), so the host is **hand-made bytecode**:
  `javac` compiles the child and this script rewrites the host's `make` method with two
  `new DBS2$1` sites, keeping the compiled host's own `InnerClasses` row and the child's
  `EnclosingMethod` intact.

Every case is frozen in both compilation legs: `v23/` from the ambient javac's `--release 8` and
`v8/` from a real javac 8 (`JARDE_JAVAC8`, defaulting to the Corretto install this repository's
evidence records). The hand-made host is generated per leg from that leg's compiled child.

    python3 tests/fixtures/proved-java-structure/double-brace-allocation-site/freeze.py
"""

from pathlib import Path
import hashlib
import os
import shutil
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent
JAVA8_HOME = Path(
    os.environ.get(
        "JARDE_JAVAC8",
        "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home",
    )
)

CONTROL = """import java.util.ArrayList;
import java.util.List;
public class DBS {
    static List<String> make(String s) { return new ArrayList<String>() {{ add(s); }}; }
    public static void main(String[] a) { System.out.println(make("z").get(0)); }
}
"""

METHODS = """import java.util.ArrayList;
import java.util.List;
public class DBM {
    static List<String> make(String s) { return new ArrayList<String>() {{ add(s); } void mark() {} }; }
    public static void main(String[] a) { System.out.println(make("z").get(0)); }
}
"""

UNSPELLABLE = """import java.util.ArrayList;
import java.util.List;
public class DBN {
    static List<String> make(String s) { return new Carrier.Nested() {{ add(s); }}; }
    public static void main(String[] a) { System.out.println(make("z").get(0)); }
}
class Carrier { static class Nested extends ArrayList<String> {} }
"""


def u1(value: int) -> bytes:
    return struct.pack(">B", value)


def u2(value: int) -> bytes:
    return struct.pack(">H", value)


def u4(value: int) -> bytes:
    return struct.pack(">I", value)


class Pool:
    """A constant pool, entry 1 always the `Code` attribute name every member here writes."""

    def __init__(self) -> None:
        self.entries: list[bytes] = [u1(1) + u2(4) + b"Code"]
        self.by_key: dict[tuple, int] = {}

    def push(self, key: tuple, data: bytes) -> int:
        if key in self.by_key:
            return self.by_key[key]
        self.entries.append(data)
        self.by_key[key] = len(self.entries)
        return len(self.entries)

    def utf8(self, text: str) -> int:
        encoded = text.encode()
        return self.push(("utf8", text), u1(1) + u2(len(encoded)) + encoded)

    def cls(self, name: int) -> int:
        return self.push(("class", name), u1(7) + u2(name))

    def nat(self, name: int, descriptor: int) -> int:
        return self.push(("nat", name, descriptor), u1(12) + u2(name) + u2(descriptor))

    def methodref(self, owner: int, name_and_type: int) -> int:
        return self.push(("methodref", owner, name_and_type), u1(10) + u2(owner) + u2(name_and_type))


def classfile(pool: Pool, flags: int, this: int, superclass: int, attributes, methods) -> bytes:
    out = b"\xca\xfe\xba\xbe" + u2(0) + u2(52)
    out += u2(len(pool.entries) + 1)
    for entry in pool.entries:
        out += entry
    out += u2(flags) + u2(this) + u2(superclass) + u2(0)
    out += u2(0)  # no fields
    out += u2(len(methods))
    for method_flags, name, descriptor, (stack, locals_, code) in methods:
        out += u2(method_flags) + u2(name) + u2(descriptor) + u2(1) + u2(1)
        attribute = u2(stack) + u2(locals_) + u4(len(code)) + code + u2(0) + u2(0)
        out += u4(len(attribute)) + attribute
    out += u2(len(attributes))
    for name, body in attributes:
        out += u2(name) + u4(len(body)) + body
    return out


def read_u2(data: bytes, at: int) -> int:
    return struct.unpack_from(">H", data, at)[0]


def read_u4(data: bytes, at: int) -> int:
    return struct.unpack_from(">I", data, at)[0]


def pool_of(data: bytes) -> tuple[list, int]:
    """The constant pool of one class file, as a list indexed by pool entry (None for a gap)."""
    count = read_u2(data, 8)
    pool: list = [None] * count
    at = 10
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            length = read_u2(data, at)
            pool[index] = ("utf8", data[at + 2 : at + 2 + length])
            at += 2 + length
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            pool[index] = ("wide", at)
            at += 4
        elif tag in (5, 6):
            pool[index] = ("wide2", at)
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pool[index] = ("short", read_u2(data, at))
            at += 2
        elif tag == 15:
            pool[index] = ("methodhandle", at)
            at += 3
        else:
            raise SystemExit(f"unknown constant pool tag {tag} at {at}")
        index += 1
    return pool, at


def class_name(pool: list, index: int) -> str:
    entry = pool[index]
    if entry[0] != "short":
        raise SystemExit(f"pool entry {index} is not a Class")
    return pool[entry[1]][1].decode()


def attributes_at(data: bytes, pool: list, at: int) -> list[tuple[str, int, int]]:
    """The `(name, body offset, length)` of every attribute of one member or class."""
    count = read_u2(data, at)
    at += 2
    found = []
    for _ in range(count):
        name = pool[read_u2(data, at)][1].decode()
        length = read_u4(data, at + 2)
        found.append((name, at + 6, length))
        at += 6 + length
    return found


def inner_classes_row(data: bytes) -> tuple[int, int, int, int]:
    """The one `InnerClasses` row of a compiled host: (class, outer, inner_name, flags)."""
    pool, at = pool_of(data)
    at += 2 + 2 + 2  # access_flags, this_class, super_class
    interfaces = read_u2(data, at)
    at += 2 + 2 * interfaces
    fields = read_u2(data, at)
    at += 2
    for _ in range(fields):
        at += 6
        count = read_u2(data, at)
        at += 2
        for _ in range(count):
            at += 6 + read_u4(data, at + 2)
    methods = read_u2(data, at)
    at += 2
    for _ in range(methods):
        at += 6
        count = read_u2(data, at)
        at += 2
        for _ in range(count):
            at += 6 + read_u4(data, at + 2)
    for name, body, _ in attributes_at(data, pool, at):
        if name == "InnerClasses":
            rows = read_u2(data, body)
            if rows != 1:
                raise SystemExit(f"the compiled host states {rows} InnerClasses rows, not one")
            return (
                class_name(pool, read_u2(data, body + 2)),
                read_u2(data, body + 4),
                read_u2(data, body + 6),
                read_u2(data, body + 8),
            )
    raise SystemExit("the compiled host declares no InnerClasses attribute")


def multi_site_host(child: bytes, host_name: str, inner_row: tuple[int, int, int, int]) -> bytes:
    """The hand-made host: two methods, each allocating the child once.

    Two *methods* rather than two statements in one, so the allocation point's own one-site
    discipline is not what refuses the shape: each method holds exactly one verified allocation,
    and the criterion under test is the owner census's single use — the class is constructed
    twice, and hiding it from the text would drop one of the two.
    """
    child_name, outer, inner_name, flags = inner_row
    if outer != 0 or inner_name != 0:
        raise SystemExit("the frozen child is not an anonymous row")
    if child_name != f"{host_name}$1":
        raise SystemExit(f"the compiled child is {child_name}, not {host_name}$1")
    pool = Pool()
    this = pool.cls(pool.utf8(host_name))
    object_class = pool.cls(pool.utf8("java/lang/Object"))
    child_class = pool.cls(pool.utf8(child_name))
    init = pool.utf8("<init>")
    object_init = pool.methodref(object_class, pool.nat(init, pool.utf8("()V")))
    child_init = pool.methodref(child_class, pool.nat(init, pool.utf8("(Ljava/lang/String;)V")))
    # `new; dup; aload_0; invokespecial child.<init>; astore_1; aload_1; areturn` — the argument
    # is the method's own parameter, exactly as the compiled child's own site passes it.
    allocate = bytes([0xBB]) + u2(child_class) + bytes([0x59, 0x2A, 0xB7]) + u2(child_init)
    site = allocate + bytes([0x4C, 0x2B, 0xB0])
    # `InnerClasses`: the compiled host's own row, so the child's own table and this one agree.
    row = u2(child_class) + u2(0) + u2(0) + u2(flags)
    constructor_code = bytes([0x2A, 0xB7]) + u2(object_init) + bytes([0xB1])
    return classfile(
        pool,
        0x0021,  # public, super
        this,
        object_class,
        [(pool.utf8("InnerClasses"), u2(1) + row)],
        [
            (0x0001, init, pool.utf8("()V"), (1, 1, constructor_code)),
            # The compiled child's own `EnclosingMethod` names this exact member.
            (0x0008, pool.utf8("make"), pool.utf8("(Ljava/lang/String;)Ljava/util/List;"), (3, 2, site)),
            (0x0008, pool.utf8("again"), pool.utf8("(Ljava/lang/String;)Ljava/lang/Object;"), (3, 2, site)),
        ],
    )


def compile_leg(directory: Path, sources: dict[str, str], javac: Path, release: bool) -> None:
    directory.mkdir(parents=True, exist_ok=True)
    for name, source in sources.items():
        (directory / f"{name}.java").write_text(source)
    command = [str(javac)]
    if release:
        command += ["--release", "8"]
    command += ["-g", "-Xlint:-options", "-d", str(directory)]
    command += [str(directory / f"{name}.java") for name in sources]
    done = subprocess.run(command, capture_output=True, check=False)
    if done.returncode != 0:
        raise SystemExit(f"{javac} rejected the fixture sources:\n{done.stderr.decode()}")


def main() -> int:
    javac8 = JAVA8_HOME / "bin" / "javac"
    if not javac8.is_file():
        raise SystemExit(f"no javac 8 at {javac8}; set JARDE_JAVAC8")
    legs = [("v23", Path(shutil.which("javac") or "javac"), True), ("v8", javac8, False)]
    for leg, javac, release in legs:
        with tempfile.TemporaryDirectory() as scratch:
            build = Path(scratch)
            compile_leg(build, {"DBS": CONTROL}, javac, release)
            compile_leg(build, {"DBM": METHODS}, javac, release)
            compile_leg(build, {"DBN": UNSPELLABLE}, javac, release)
            compile_leg(build, {"DBS2": CONTROL.replace("DBS", "DBS2")}, javac, release)
            for case, names in [
                ("controls/single-site", ["DBS.class", "DBS$1.class"]),
                ("negatives/methods", ["DBM.class", "DBM$1.class"]),
                (
                    "negatives/unspellable-super",
                    ["DBN.class", "DBN$1.class", "Carrier.class", "Carrier$Nested.class"],
                ),
            ]:
                target = ROOT / leg / case
                target.mkdir(parents=True, exist_ok=True)
                for name in names:
                    shutil.copyfile(build / name, target / name)
            # The multi-site host: the compiled child, and the hand-made host that allocates it
            # twice. The host's own `InnerClasses` row is read out of the compiled host first.
            compiled_host = (build / "DBS2.class").read_bytes()
            child = (build / "DBS2$1.class").read_bytes()
            target = ROOT / leg / "negatives/multi-site"
            target.mkdir(parents=True, exist_ok=True)
            (target / "DBS2$1.class").write_bytes(child)
            (target / "DBS2.class").write_bytes(
                multi_site_host(child, "DBS2", inner_classes_row(compiled_host))
            )
    lines = []
    for path in sorted(ROOT.rglob("*.class")):
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {path.relative_to(ROOT)}")
    (ROOT / "SHA256SUMS").write_text("\n".join(lines) + "\n")
    print(f"froze {len(lines)} class file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
