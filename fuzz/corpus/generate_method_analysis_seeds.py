#!/usr/bin/env python3
"""Deterministic generator for the P2 method-analysis fuzz corpus (design 5.3).

Every seed is a small artifact whose *first member with a body* is the shape the corpus is
meant to cover, because the method-analysis target derives the method it analyzes from the
input (the first member of the first class whose header it can read that declares a `Code`
attribute). The shapes the slice names:

    corpus/method_analysis/jsr-ret.class           the committed `jsr`/`ret` fixture, copied
                                                  byte for byte from the historical corpus
    corpus/method_analysis/legacy-clone.class      one shared subroutine called from two sites
    corpus/method_analysis/wide-switch.class       `lookupswitch` + `tableswitch` + `wide`
                                                  local access in one body
    corpus/method_analysis/exception-overlap.class three overlapping exception records, each
                                                  handler entered with one exception type
    corpus/method_analysis/exception-overlap-mixed.class
                                                  the same table with a handler entered with
                                                  a typed and a catch-all record
    corpus/method_analysis/wide-switch.jar         the wide/switch body as a `Base` entry of
                                                  a ZIP, which is the derivation path a
                                                  standalone class root does not reach

The `jsr-ret` seed is the `ecj-4.6.1/v45` fixture: the same source the reader, call-context
and canonical-CFG tests use, whose `finallyPath(I)I` really compiles `finally` into two `jsr`
calls of one subroutine — the shape the clone normalization exists for. Its first member with
a body is `<init>`, so the driver derives that one; the corpus test asks for `finallyPath`
explicitly. The other three seeds are written here from scratch: `legacy-clone` uses the
`jsr` era dialect (major 50, below the major 51 the modern dialect starts at) and the other
two are modern (major 52).

Regenerate (writes `corpus/method_analysis/`, prints the digest of every file):

    python3 fuzz/corpus/generate_method_analysis_seeds.py

Nothing here drives the fuzzer: cargo-fuzz mutates these seeds itself. The fuzz targets
never call this script, and the smoke gate never needs it.
"""

from __future__ import annotations

import hashlib
import io
import pathlib
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
FIXTURE = HERE.parent.parent / "tests" / "fixtures" / "historical" / "ecj-4.6.1"


def u2(value: int) -> bytes:
    return value.to_bytes(2, "big")


def u4(value: int) -> bytes:
    return value.to_bytes(4, "big")


def i2(value: int) -> bytes:
    return value.to_bytes(2, "big", signed=True)


def i4(value: int) -> bytes:
    return value.to_bytes(4, "big", signed=True)


class Pool:
    """Constant pool builder; indexes are 1-based like the class file format."""

    def __init__(self) -> None:
        self.entries: list[bytes] = []

    def add(self, entry: bytes) -> int:
        self.entries.append(entry)
        return len(self.entries)

    def utf8(self, text: bytes) -> int:
        return self.add(b"\x01" + u2(len(text)) + text)

    def class_(self, name_index: int) -> int:
        return self.add(b"\x07" + u2(name_index))

    def bytes(self) -> bytes:
        return b"".join(self.entries)


class Body:
    """A tiny two-pass assembler for the hand-written bodies of the seeds.

    Branches and switches name labels instead of raw offsets, so a body reads like the JVMS
    listing it is: the first pass lays the instructions out and records every label's BCI, the
    second emits the branch offsets and the `tableswitch`/`lookupswitch` payloads, whose
    padding depends on the BCI the opcode itself sits at.
    """

    def __init__(self) -> None:
        self.items: list[tuple] = []

    def label(self, name: str) -> None:
        self.items.append(("label", name))

    def insn(self, *opcode: int) -> None:
        """One instruction whose bytes are already complete (no label operand)."""
        self.items.append(("bytes", bytes(opcode)))

    def branch(self, opcode: int, target: str) -> None:
        """A 2-byte relative branch; the offset is resolved from `target`."""
        self.items.append(("branch", opcode, target))

    def tableswitch(self, default: str, low: int, targets: list[str]) -> None:
        self.items.append(("tableswitch", default, low, targets))

    def lookupswitch(self, default: str, pairs: list[tuple[int, str]]) -> None:
        self.items.append(("lookupswitch", default, pairs))

    def assemble(self) -> bytes:
        offsets: dict[str, int] = {}
        position = 0
        for item in self.items:
            kind = item[0]
            if kind == "label":
                offsets[item[1]] = position
            elif kind == "bytes":
                position += len(item[1])
            elif kind == "branch":
                position += 3
            else:
                pad = (4 - ((position + 1) % 4)) % 4
                if kind == "tableswitch":
                    # opcode, padding, default, low, high, one offset per entry
                    position += 1 + pad + 4 + 4 + 4 + 4 * len(item[3])
                else:
                    # opcode, padding, default, pair count, one key/offset pair each
                    position += 1 + pad + 4 + 4 + 8 * len(item[2])

        code = bytearray()
        for item in self.items:
            kind = item[0]
            if kind == "label":
                continue
            if kind == "bytes":
                code += item[1]
                continue
            here = len(code)
            if kind == "branch":
                code.append(item[1])
                code += i2(offsets[item[2]] - here)
                continue
            pad = (4 - ((here + 1) % 4)) % 4
            if kind == "tableswitch":
                default, low, targets = item[1], item[2], item[3]
                code.append(0xAA)
                code += b"\x00" * pad
                code += i4(offsets[default] - here)
                code += i4(low)
                code += i4(low + len(targets) - 1)
                for target in targets:
                    code += i4(offsets[target] - here)
            else:
                default, pairs = item[1], item[2]
                code.append(0xAB)
                code += b"\x00" * pad
                code += i4(offsets[default] - here)
                code += i4(len(pairs))
                for key, target in pairs:
                    code += i4(key) + i4(offsets[target] - here)
        self.offsets = offsets
        return bytes(code)


def class_bytes(this_class: bytes, major: int, methods: list[dict]) -> bytes:
    """A structurally valid class file with the given methods and nothing else.

    One method is `{"access", "name", "descriptor", "max_stack", "max_locals", "code"}` plus
    an optional `"handlers"`: `(start_bci, end_bci, handler_bci, catch_class_or_None)` records,
    with `None` for a catch-all. The `Code` attribute of every method carries no attribute of
    its own, so the seeds stay minimal and every byte is accounted for here.
    """
    pool = Pool()
    this_name = pool.utf8(this_class)
    this_index = pool.class_(this_name)
    super_index = pool.class_(pool.utf8(b"java/lang/Object"))
    code_attribute = pool.utf8(b"Code")

    entries = []
    for method in methods:
        name = pool.utf8(method["name"])
        descriptor = pool.utf8(method["descriptor"])
        code = method["code"]
        body = bytearray()
        body += u2(method["max_stack"]) + u2(method["max_locals"])
        body += u4(len(code)) + code
        handlers = method.get("handlers", [])
        body += u2(len(handlers))
        for start, end, handler, catch in handlers:
            catch_index = 0
            if catch is not None:
                catch_index = pool.class_(pool.utf8(catch))
            body += u2(start) + u2(end) + u2(handler) + u2(catch_index)
        body += u2(0)  # no attributes of the `Code` attribute
        entries.append(
            u2(method["access"])
            + u2(name)
            + u2(descriptor)
            + u2(1)
            + u2(code_attribute)
            + u4(len(body))
            + bytes(body)
        )

    return (
        u4(0xCAFEBABE)
        + u2(0)  # minor
        + u2(major)
        + u2(len(pool.entries) + 1)
        + pool.bytes()
        + u2(0x0021)  # ACC_PUBLIC | ACC_SUPER
        + u2(this_index)
        + u2(super_index)
        + u2(0)  # interfaces
        + u2(0)  # fields
        + u2(len(entries))
        + b"".join(entries)
        + u2(0)  # class attributes
    )


def jsr_ret() -> bytes:
    """The committed `jsr`/`ret` fixture, copied byte for byte.

    The bytes are read from `tests/fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class`
    and written into the corpus: the fixture itself is not modified, not linked and not
    regenerated, so the seed is the same artifact the P2 tests analyze.
    """
    return (FIXTURE / "v45" / "HistoricalControlFlow.class").read_bytes()


def legacy_clone() -> bytes:
    """One subroutine body shared by two `jsr` call sites.

    The body is the minimal shape of a compiled `finally` in the legacy dialect: two call
    sites push their return address into the shared subroutine, whose body stores the address
    in a local and returns through it. The clone normalization exists to give each call site
    its own copy, so this seed is the smallest input that must charge `NormalizationClones`.

    ```text
    BCI  0: jsr +7 -> 7      (return address 3)
    BCI  3: jsr +4 -> 7      (return address 6)
    BCI  6: return
    BCI  7: astore_0         (the shared subroutine stores the address)
    BCI  8: ret 0
    ```
    """
    body = Body()
    body.branch(0xA8, "sub")  # jsr
    body.branch(0xA8, "sub")  # jsr
    body.insn(0xB1)  # return
    body.label("sub")
    body.insn(0x4B)  # astore_0
    body.insn(0xA9, 0x00)  # ret 0
    return class_bytes(
        b"fuzz/SharedSubroutine",
        50,  # the legacy dialect: `jsr`/`ret` are forbidden from major 51 on
        [
            {
                "access": 0x0009,  # ACC_PUBLIC | ACC_STATIC
                "name": b"shared",
                "descriptor": b"()V",
                "max_stack": 1,
                "max_locals": 1,
                "code": body.assemble(),
            }
        ],
    )


def wide_switch() -> bytes:
    """One body with a `lookupswitch`, a `tableswitch` and `wide` local access.

    Both switch opcodes are reachable and every case target is a real block: the
    `lookupswitch` keys 10 and 20 and its default each push a distinct value, the three
    branches merge, `wide istore 300`/`wide iload 300` write and read a local no short form can
    name, and the `tableswitch` over that value reaches all three of its cases.

    ```text
    BCI  0: iload_0
    BCI  1: lookupswitch      default -> push0, 10 -> push1, 20 -> push2
    push1: iconst_1; goto merge
    push2: iconst_2; goto merge
    push0: iconst_0; goto merge
    merge: wide istore 300; wide iload 300
           tableswitch         default -> zero, 0..2 -> one, two, three
    one: iconst_1; ireturn    two: iconst_2; ireturn    three: iconst_3; ireturn
    zero: iconst_0; ireturn
    ```
    """
    body = Body()
    body.insn(0x1A)  # iload_0
    body.lookupswitch("push0", [(10, "push1"), (20, "push2")])
    body.label("push1")
    body.insn(0x04)  # iconst_1
    body.branch(0xA7, "merge")  # goto
    body.label("push2")
    body.insn(0x05)  # iconst_2
    body.branch(0xA7, "merge")
    body.label("push0")
    body.insn(0x03)  # iconst_0
    body.label("merge")
    body.insn(0xC4, 0x36, 0x01, 0x2C)  # wide istore 300
    body.insn(0xC4, 0x15, 0x01, 0x2C)  # wide iload 300
    body.tableswitch("zero", 0, ["one", "two", "three"])
    body.label("one")
    body.insn(0x04, 0xAC)  # iconst_1; ireturn
    body.label("two")
    body.insn(0x05, 0xAC)  # iconst_2; ireturn
    body.label("three")
    body.insn(0x06, 0xAC)  # iconst_3; ireturn
    body.label("zero")
    body.insn(0x03, 0xAC)  # iconst_0; ireturn
    return class_bytes(
        b"fuzz/WideSwitch",
        52,
        [
            {
                "access": 0x0009,  # ACC_PUBLIC | ACC_STATIC
                "name": b"pick",
                "descriptor": b"(I)I",
                "max_stack": 1,
                "max_locals": 301,
                "code": body.assemble(),
            }
        ],
    )


def overlap_body(mixed: bool) -> tuple[bytes, list[tuple]]:
    """The body and exception table shared by the two overlap seeds.

    The protected ranges are deliberately not a partition: `[0, 6)` names one handler,
    `[4, 9)` a second one, and `[0, 9)` overlaps both, so the throw site at BCI 5 is covered
    by three records at once and the two handlers are reachable through two records each. The
    handler BCIs come from the labels themselves, so the table cannot drift away from the code
    it protects.

    ```text
    BCI  0: iload_0
    BCI  1: ifeq +8 -> 9
    BCI  4: aconst_null
    BCI  5: athrow
    BCI  6: nop, 7: nop, 8: nop
    BCI  9: iconst_1; ireturn
    BCI 11: pop; iconst_2; ireturn      (the first handler)
    BCI 14: pop; iconst_3; ireturn      (the second handler)
    ```

    `mixed` decides what the first handler is entered with: with `mixed=False` both of its
    records carry the same catch type, so the handler's entry state is one type; with
    `mixed=True` one of them is a catch-all and the other is typed, so the same handler is
    entered with two different exception types and the seeds cover both merges.
    """
    body = Body()
    body.insn(0x1A)  # iload_0
    body.branch(0x99, "zero")  # ifeq
    body.insn(0x01)  # aconst_null
    body.insn(0xBF)  # athrow
    body.insn(0x00, 0x00, 0x00)  # three nops: the range end at BCI 9 must be an instruction
    body.label("zero")
    body.insn(0x04, 0xAC)  # iconst_1; ireturn
    body.label("handler_one")
    body.insn(0x57, 0x05, 0xAC)  # pop; iconst_2; ireturn
    body.label("handler_two")
    body.insn(0x57, 0x06, 0xAC)  # pop; iconst_3; ireturn
    code = body.assemble()
    one = body.offsets["handler_one"]
    two = body.offsets["handler_two"]
    handlers = [
        (0, 6, one, b"java/lang/Throwable"),
        (4, 9, two, None),
        (0, 9, one, None if mixed else b"java/lang/Throwable"),
    ]
    return code, handlers


def overlap_class(code: bytes, handlers: list[tuple]) -> bytes:
    return class_bytes(
        b"fuzz/ExceptionOverlap",
        52,
        [
            {
                "access": 0x0009,  # ACC_PUBLIC | ACC_STATIC
                "name": b"guarded",
                "descriptor": b"(I)I",
                "max_stack": 1,
                "max_locals": 1,
                "code": code,
                "handlers": handlers,
            }
        ],
    )


def exception_overlap() -> bytes:
    """The overlap whose two handlers are each entered with one exception type."""
    code, handlers = overlap_body(mixed=False)
    return overlap_class(code, handlers)


def exception_overlap_mixed() -> bytes:
    """The overlap whose first handler is entered with a typed and an untyped record.

    Both seeds are legal class files and both are what the corpus needs: this one is the
    shape where a handler's entry state really merges two different exception types, which is
    the part of the table the frame and SSA phases have to agree about.
    """
    code, handlers = overlap_body(mixed=True)
    return overlap_class(code, handlers)


def wide_switch_jar() -> bytes:
    """The wide/switch class inside a ZIP, so the entry-probing derivation is seeded too.

    The driver derives a driver method from a standalone class root and from a `Base` class
    entry of a `ZIP`; this seed is the second path, with the same body as the standalone
    `wide-switch` seed, so the zip path has a body that reaches the end of the pipeline.
    """
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
        info = zipfile.ZipInfo("fuzz/WideSwitch.class", date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_STORED
        info.create_system = 0
        info.external_attr = 0
        archive.writestr(info, wide_switch())
    return buffer.getvalue()


SEEDS: dict[str, bytes] = {
    "jsr-ret.class": jsr_ret(),
    "legacy-clone.class": legacy_clone(),
    "wide-switch.class": wide_switch(),
    "exception-overlap.class": exception_overlap(),
    "exception-overlap-mixed.class": exception_overlap_mixed(),
    "wide-switch.jar": wide_switch_jar(),
}


def main() -> None:
    directory = HERE / "method_analysis"
    directory.mkdir(parents=True, exist_ok=True)
    for name, data in SEEDS.items():
        (directory / name).write_bytes(data)
        print(f"{hashlib.sha256(data).hexdigest()}  {len(data):>6}  method_analysis/{name}")


if __name__ == "__main__":
    main()
