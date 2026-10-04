#!/usr/bin/env python3
"""Derive the six mixed-form refusal negative classes and record their behavior baselines.

The frozen fixture `anonymous-super-mixed-direct` is the positive anchor. Six inputs must keep
physical class text because the proved parameter partition (or the one-field / one-site shape)
refuses them. Two are expressible in javac source (frozen beside this script's inputs); four are
minimal classfile patches of the anchor child, derived here exactly as the CI test
`anonymous_superclass_refuses_unproved_mixed_parameter_roles` derives them in memory:

  unconsumed   nop out the capture load+store (aload_3; putfield)      -> parameter has no proved consumption
  dual-role    aload_1 -> aload_3 at the invoke's first argument        -> one parameter feeds both roles
  reordered    swap aload_1/iload_2 and rewrite the invoked descriptor  -> super arguments leave physical order
  double-write replace the closing return with a second putfield+return -> the capture field is stored twice

Each patched class must verify and run under `java -Xverify:all` before it is used as a negative,
and every Jarde rendering must keep `new ...$1(...)` physical text.
"""

import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
ANCHOR = REPO / "tests/fixtures/proved-java-structure/anonymous-super-mixed-direct"
CLI = REPO / "target/debug/jarde-cli"
OUT = HERE / "negative-derivations"


def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, text=True, **kwargs)
    return result


def require(result, what):
    if result.returncode != 0:
        print(f"{what} FAILED (exit {result.returncode})\n{result.stdout}\n{result.stderr}")
        raise SystemExit(1)
    return result


def utf8_patches(data: bytes, old: bytes, new: bytes) -> bytes:
    """Rewrite one UTF8 constant in place; the two spellings must be equally long."""
    assert len(old) == len(new)
    count = int.from_bytes(data[8:10], "big")
    cursor = 10
    index = 1
    hits = 0
    while index < count:
        tag = data[cursor]
        if tag == 1:
            length = int.from_bytes(data[cursor + 1 : cursor + 3], "big")
            if data[cursor + 3 : cursor + 3 + length] == old:
                data[cursor + 3 : cursor + 3 + length] = new
                hits += 1
            cursor += 3 + length
            index += 1
        elif tag in (3, 4):
            cursor += 5
            index += 1
        elif tag in (5, 6):
            cursor += 9
            index += 2
        elif tag in (7, 8, 13, 16, 19, 20):
            cursor += 3
            index += 1
        elif tag == 15:
            cursor += 4
            index += 1
        elif tag in (9, 10, 11, 12, 18):
            cursor += 5
            index += 1
        else:
            raise AssertionError(f"unknown cp tag {tag}")
    assert hits == 1, f"expected exactly one {old!r} constant, found {hits}"
    return data


def child_patches(child: bytes):
    """The anchor child's constructor is aload_0; aload_3; putfield; aload_0; aload_1;
    iload_2; invokespecial; return. Return the four patched variants."""
    # locate the Code attribute of <init> by walking the class like the Rust test helpers do
    def u2(data, off):
        return int.from_bytes(data[off : off + 2], "big")

    def u4(data, off):
        return int.from_bytes(data[off : off + 4], "big")

    count = u2(child, 8)
    cursor = 10
    index = 1
    utf8 = [b""] * count
    while index < count:
        tag = child[cursor]
        if tag == 1:
            length = u2(child, cursor + 1)
            utf8[index] = child[cursor + 3 : cursor + 3 + length]
            cursor += 3 + length
            index += 1
        elif tag in (3, 4):
            cursor += 5
            index += 1
        elif tag in (5, 6):
            cursor += 9
            index += 2
        elif tag in (7, 8, 16, 19, 20):
            cursor += 3
            index += 1
        elif tag in (9, 10, 11, 12, 17, 18):
            cursor += 5
            index += 1
        elif tag == 15:
            cursor += 4
            index += 1
        else:
            raise AssertionError(f"unknown cp tag {tag}")
    # access_flags, this_class, super_class, interfaces_count and the interfaces themselves
    cursor += 6
    cursor += 2 + 2 * u2(child, cursor)
    # fields
    field_count = u2(child, cursor)
    cursor += 2
    for _ in range(field_count):
        cursor += 6
        attribute_count_offset = cursor
        cursor += 2
        for _ in range(u2(child, attribute_count_offset)):
            length = u4(child, cursor + 2)
            cursor += 6 + length
    # methods
    method_count = u2(child, cursor)
    cursor += 2
    code_offset = None
    attribute_length_offset = None
    code_length_offset = None
    for _ in range(method_count):
        cursor += 6
        attribute_count_offset = cursor
        cursor += 2
        for _ in range(u2(child, attribute_count_offset)):
            name_index = u2(child, cursor)
            length_offset = cursor + 2
            data_offset = length_offset + 4
            length = u4(child, length_offset)
            if utf8[name_index] == b"Code" and child[data_offset + 8] == 0x2A and child[data_offset + 9] == 0x2D:
                code_offset = data_offset + 8
                attribute_length_offset = length_offset
                code_length_offset = data_offset + 4
            cursor = data_offset + length
    assert code_offset is not None, "the anonymous constructor's Code attribute was not found"
    shape = bytes([0x2A, 0x2D, 0xB5, 0x00, 0x01, 0x2A, 0x2B, 0x1C, 0xB7])
    assert child[code_offset : code_offset + 9] == shape, child[code_offset : code_offset + 12].hex()

    patches = {}

    unconsumed = bytearray(child)
    for offset in range(0, 5):
        unconsumed[code_offset + offset] = 0x00
    patches["unconsumed-param"] = bytes(unconsumed)

    dual = bytearray(child)
    assert dual[code_offset + 6] == 0x2B
    dual[code_offset + 6] = 0x2D
    patches["dual-role-param"] = bytes(dual)

    reordered = bytearray(child)
    reordered[code_offset + 6], reordered[code_offset + 7] = (
        reordered[code_offset + 7],
        reordered[code_offset + 6],
    )
    utf8_patches(
        reordered,
        b"(Ljava/lang/String;I)V",
        b"(ILjava/lang/String;)V",
    )
    patches["reordered-super-args"] = bytes(reordered)

    double_write = bytearray(child)
    putfield_index = child[code_offset + 3 : code_offset + 5]
    tail = bytes([0x2A, 0x2D]) + putfield_index + bytes([0xB1])
    assert u4(double_write, code_length_offset) == 12
    double_write[code_offset + 11 : code_offset + 12] = tail
    double_write[code_length_offset : code_length_offset + 4] = (16).to_bytes(4, "big")
    double_write[attribute_length_offset : attribute_length_offset + 4] = (
        u4(double_write, attribute_length_offset) + 4
    ).to_bytes(4, "big")
    patches["double-write"] = bytes(double_write)

    return patches


def class_source_of(jar, class_name):
    result = run(
        [
            str(CLI),
            "class-source",
            "--input",
            str(jar),
            "--class",
            class_name,
            "--policy",
            "plain-jar",
            "--release",
            "8",
            "--format",
            "text",
        ]
    )
    return result


def main():
    OUT.mkdir(exist_ok=True)
    work = pathlib.Path(tempfile.mkdtemp(prefix="mixed-refusals-"))
    try:
        # rebuild the anchor so the derivation starts from the frozen sources
        require(
            run(
                [
                    "javac",
                    "--release",
                    "8",
                    "-g:none",
                    "-d",
                    str(work),
                    str(ANCHOR / "AnonymousSuperMixedDirect.java"),
                ]
            ),
            "anchor compile",
        )
        root_bytes = (work / "AnonymousSuperMixedDirect.class").read_bytes()
        child_bytes = (work / "AnonymousSuperMixedDirect$1.class").read_bytes()
        base_bytes = (work / "Base.class").read_bytes()
        refusals = REPO / "tests/fixtures/proved-java-structure/anonymous-super-mixed-refusals"
        two_captures = refusals / "two-capture-fields"
        two_sites = refusals / "two-mixed-sites"
        for name in ("TwoCaptureFields", "TwoMixedSites"):
            require(
                run(
                    [
                        "javac",
                        "--release",
                        "8",
                        "-g:none",
                        "-d",
                        str({"TwoCaptureFields": two_captures, "TwoMixedSites": two_sites}[name]),
                        str({"TwoCaptureFields": two_captures, "TwoMixedSites": two_sites}[name] / f"{name}.java"),
                    ]
                ),
                f"{name} compile",
            )
        # TwoMixedSites' multi-site variant: the second site's class constant names the first child
        sites_root = bytearray((two_sites / "TwoMixedSites.class").read_bytes())

        def u2(data, off):
            return int.from_bytes(data[off : off + 2], "big")

        count = u2(sites_root, 8)
        cursor = 10
        index = 1
        utf8 = [b""] * count
        class_entries = {}
        while index < count:
            tag = sites_root[cursor]
            if tag == 1:
                length = u2(sites_root, cursor + 1)
                utf8[index] = sites_root[cursor + 3 : cursor + 3 + length]
                cursor += 3 + length
                index += 1
            elif tag == 7:
                class_entries[index] = u2(sites_root, cursor + 1)
                cursor += 3
                index += 1
            elif tag in (3, 4):
                cursor += 5
                index += 1
            elif tag in (5, 6):
                cursor += 9
                index += 2
            elif tag in (8, 16, 19, 20):
                cursor += 3
                index += 1
            elif tag in (9, 10, 11, 12, 17, 18):
                cursor += 5
                index += 1
            elif tag == 15:
                cursor += 4
                index += 1
            else:
                raise AssertionError(f"unknown cp tag {tag}")
        first_name_index = next(
            i for i, content in enumerate(utf8) if content == b"TwoMixedSites$1"
        )
        second_class_index = next(
            i for i, name_index in class_entries.items() if utf8[name_index] == b"TwoMixedSites$2"
        )
        entry_offset = 10
        cursor = 10
        index = 1
        while index < count:
            if index == second_class_index:
                break
            tag = sites_root[cursor]
            if tag == 1:
                cursor += 3 + u2(sites_root, cursor + 1)
                index += 1
            elif tag in (3, 4):
                cursor += 5
                index += 1
            elif tag in (5, 6):
                cursor += 9
                index += 2
            elif tag in (7, 8, 16, 19, 20):
                cursor += 3
                index += 1
            elif tag in (9, 10, 11, 12, 17, 18):
                cursor += 5
                index += 1
            elif tag == 15:
                cursor += 4
                index += 1
            else:
                raise AssertionError(f"unknown cp tag {tag}")
        assert sites_root[cursor] == 7
        sites_root[cursor + 1 : cursor + 3] = first_name_index.to_bytes(2, "big")
        (OUT / "TwoMixedSites-multi-site-patched.class").write_bytes(bytes(sites_root))

        cases = {}
        for label, patched in child_patches(child_bytes).items():
            cases[label] = {
                "root": root_bytes,
                "child": patched,
                "base": base_bytes,
                "root_name": "AnonymousSuperMixedDirect",
            }
        cases["multi-site"] = {
            "root": bytes(sites_root),
            "child": (two_sites / "TwoMixedSites$1.class").read_bytes(),
            "base": (two_sites / "Base.class").read_bytes(),
            "root_name": "TwoMixedSites",
            "extra_child": (two_sites / "TwoMixedSites$2.class").read_bytes(),
            "extra_child_name": "TwoMixedSites$2",
        }
        cases["two-capture-fields"] = {
            "root": (two_captures / "TwoCaptureFields.class").read_bytes(),
            "child": (two_captures / "TwoCaptureFields$1.class").read_bytes(),
            "base": (two_captures / "Base.class").read_bytes(),
            "root_name": "TwoCaptureFields",
        }

        report = []
        for label, case in cases.items():
            directory = work / label
            directory.mkdir()
            (directory / f"{case['root_name']}.class").write_bytes(case["root"])
            (directory / f"{case['root_name']}$1.class").write_bytes(case["child"])
            (directory / "Base.class").write_bytes(case["base"])
            if "extra_child" in case:
                (directory / f"{case['extra_child_name']}.class").write_bytes(case["extra_child"])
            packed = run(
                ["jar", "--create", "--file", str(directory / "in.jar"), "-C", str(directory), "."]
            )
            require(packed, f"{label} pack")
            # the negative must itself be a valid, runnable class
            executed = run(
                ["java", "-Xverify:all", "-cp", str(directory), case["root_name"]]
            )
            if executed.returncode != 0:
                print(f"{label}: java -Xverify:all FAILED\n{executed.stdout}\n{executed.stderr}")
                raise SystemExit(1)
            rendered = class_source_of(directory / "in.jar", case["root_name"])
            # a refusal exits nonzero and prints the JSON reason on stderr; the stdout text is
            # the physical presentation the refusal keeps
            (OUT / f"{label}-original-run.log").write_text(executed.stdout)
            (OUT / f"{label}-jarde-physical.java").write_text(rendered.stdout)
            (OUT / f"{label}-cli-error.json").write_text(rendered.stderr)
            if "new Base((" in rendered.stdout:
                print(f"{label}: the root text projected, it must stay physical")
                raise SystemExit(1)
            reason = ""
            structured = run(
                [
                    str(CLI),
                    "class-source",
                    "--input",
                    str(directory / "in.jar"),
                    "--class",
                    case["root_name"],
                    "--policy",
                    "plain-jar",
                    "--release",
                    "8",
                    "--format",
                    "json",
                ]
            )
            try:
                document = json.loads(structured.stdout)
                projection = document.get("anonymous_interface_projection", {})
                reason = str(projection.get("reason", ""))[:160]
            except json.JSONDecodeError:
                reason = "(no JSON report)"
            digest = hashlib.sha256(
                b"".join(
                    sorted(
                        path.read_bytes()
                        for path in directory.glob("*.class")
                    )
                )
            ).hexdigest()
            report.append(
                f"{label}: physical text kept; refusal reason: {reason}; "
                f"class digests sha256 {digest}"
            )

        (OUT / "SUMMARY.txt").write_text("\n".join(report) + "\n")
        print("\n".join(report))
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
