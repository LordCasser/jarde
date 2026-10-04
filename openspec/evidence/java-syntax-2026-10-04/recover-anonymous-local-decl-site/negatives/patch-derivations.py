#!/usr/bin/env python3
"""Derive this slice's patch-based refusal inputs and record Jarde's routing for each.

Two inputs cannot be expressed in javac source and are minimal classfile patches, derived here
exactly as the CI test in `tests/class_source.rs` derives them in memory:

  non-unique-alloc  retarget the second declaration site's class constant (`TwoDeclSites$2`) to
                    the first child's name, so both initializer sites allocate one physical class
                    and the single-allocation proof cannot close. Both children share the same
                    constructor shape, so the patched root still verifies and runs.
  self-alloc        retarget the anchor child's `java/lang/StringBuilder` class constant to the
                    child's own name, so the anonymous body allocates the anonymous class itself.
                    The patched child is Jarde-readable but deliberately NOT JVM-verifiable (the
                    copied `<init>:()V` ref does not resolve to the real constructor), so it has
                    no run leg: the measurement is Jarde's routing, recorded verbatim.

The source-level negatives beside this script (`nested-super-parent`, `two-decl-sites`,
`unresolvable-child-read`, `unspellable-owner-alloc`, `nested-anon-alloc`) run and verify as-is;
their baselines are recorded next to the outputs here.
"""

import pathlib
import shutil
import subprocess
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[4]
FIXTURES = REPO / "tests/fixtures/proved-java-structure"
CLI = pathlib.Path(str(REPO)) / "target/debug/jarde-cli"
OUT = HERE


def run(command, **kwargs):
    return subprocess.run(command, capture_output=True, text=True, **kwargs)


def classes_of(path: pathlib.Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.glob("*.class"))}


def retarget_class_entry_two_pass(data: bytes, old_name: bytes, new_name: bytes) -> bytes:
    count = int.from_bytes(data[8:10], "big")
    # pass 1: index every Utf8
    utf8: dict[int, bytes] = {}
    class_entries: list[tuple[int, int]] = []  # (cp_index, name_index)
    cursor = 10
    index = 1
    while index < count:
        tag = data[cursor]
        if tag == 1:
            length = int.from_bytes(data[cursor + 1 : cursor + 3], "big")
            utf8[index] = bytes(data[cursor + 3 : cursor + 3 + length])
            cursor += 3 + length
            index += 1
        elif tag == 7:
            class_entries.append((index, int.from_bytes(data[cursor + 1 : cursor + 3], "big")))
            cursor += 3
            index += 1
        elif tag in (3, 4):
            cursor += 5
            index += 1
        elif tag in (5, 6):
            cursor += 9
            index += 2
        elif tag in (8, 13, 16, 19, 20):
            cursor += 3
            index += 1
        elif tag in (9, 10, 11, 12, 17, 18):
            cursor += 5
            index += 1
        elif tag == 15:
            cursor += 4
            index += 1
        else:
            raise SystemExit(f"unknown constant tag {tag} at {cursor}")
    new_index = next((i for i, raw in utf8.items() if raw == new_name), None)
    if new_index is None:
        raise SystemExit(f"{new_name!r} is not in the pool")
    hits = [(cp, name) for cp, name in class_entries if utf8.get(name) == old_name]
    if len(hits) != 1:
        raise SystemExit(f"expected exactly one Class entry for {old_name!r}, found {len(hits)}")
    cp_index, _ = hits[0]
    offset = 10
    cursor = 10
    index = 1
    while index < count:
        tag = data[cursor]
        if index == cp_index and tag == 7:
            break
        if tag == 1:
            cursor += 3 + int.from_bytes(data[cursor + 1 : cursor + 3], "big")
            index += 1
        elif tag == 7:
            cursor += 3
            index += 1
        elif tag in (3, 4):
            cursor += 5
            index += 1
        elif tag in (5, 6):
            cursor += 9
            index += 2
        elif tag in (8, 13, 16, 19, 20):
            cursor += 3
            index += 1
        elif tag in (9, 10, 11, 12, 17, 18):
            cursor += 5
            index += 1
        elif tag == 15:
            cursor += 4
            index += 1
        else:
            raise SystemExit(f"unknown constant tag {tag} at {cursor}")
    data[cursor + 1 : cursor + 3] = new_index.to_bytes(2, "big")
    return data


def render(classes: dict[str, bytes], root: str, label: str) -> tuple[int, str, str]:
    with tempfile.TemporaryDirectory() as work:
        work_dir = pathlib.Path(work)
        for name, raw in classes.items():
            (work_dir / name).write_bytes(raw)
        packed = run(["jar", "--create", "--file", "f.jar", "-C", ".", "."], cwd=work_dir)
        if packed.returncode:
            raise SystemExit(f"jar failed: {packed.stderr}")
        request = run(
            [str(CLI), "class-source", "--input", "f.jar", "--class", root,
             "--policy", "plain-jar", "--release", "8", "--format", "text"],
            cwd=work_dir,
        )
    (OUT / f"{label}.java").write_text(request.stdout)
    (OUT / f"{label}.err").write_text(request.stderr)
    return request.returncode, request.stdout, request.stderr


def main() -> None:
    # --- non-unique-alloc: two initializer sites, one physical class -------------------------
    source = FIXTURES / "anonymous-local-decl-site-refusals/two-decl-sites"
    classes = classes_of(source)
    patched = retarget_class_entry_two_pass(
        bytearray(classes["TwoDeclSites.class"]), b"TwoDeclSites$2", b"TwoDeclSites$1"
    )
    classes["TwoDeclSites.class"] = bytes(patched)
    code, text, _ = render(classes, "TwoDeclSites", "non-unique-alloc")
    (OUT / "non-unique-alloc.exit").write_text(f"{code}\n")
    physical = "new TwoDeclSites$1((java.lang.String) text(" in text
    projected = "new Base(" in text
    (OUT / "non-unique-alloc.routing.txt").write_text(
        f"exit={code}\nphysical_presentations_of_patched_site={text.count('new TwoDeclSites$1((java.lang.String) text(')}\n"
        f"projected_new_Base={projected}\nkept_physical={physical and not projected}\n"
    )
    print(f"non-unique-alloc: exit={code} physical={physical} projected={projected}")

    # --- self-alloc: the anchor child's StringBuilder class retargeted to itself -------------
    source = FIXTURES / "anonymous-super-args"
    classes = classes_of(source)
    patched_child = retarget_class_entry_two_pass(
        bytearray(classes["AnonymousSuperArgs$1.class"]),
        b"java/lang/StringBuilder",
        b"AnonymousSuperArgs$1",
    )
    classes["AnonymousSuperArgs$1.class"] = bytes(patched_child)
    (OUT / "self-alloc-child-patched.class").write_bytes(bytes(patched_child))
    code, text, _ = render(classes, "AnonymousSuperArgs", "self-alloc")
    (OUT / "self-alloc.exit").write_text(f"{code}\n")
    child_view = run([
        str(CLI), "class-source", "--input",
        str(pathlib.Path(str(OUT / "self-alloc-child-patched.class"))),
        "--class", "AnonymousSuperArgs$1",
        "--policy", "single-class", "--release", "8", "--format", "text",
    ])
    (OUT / "self-alloc-child.java").write_text(child_view.stdout)
    (OUT / "self-alloc.routing.txt").write_text(
        f"root_request_exit={code}\nroot_kept_physical="
        f"{'AnonymousSuperArgs$1 local2 = new AnonymousSuperArgs$1(' in text}\n"
        f"root_projected={'new Base(' in text}\n"
    )
    print(
        f"self-alloc: root exit={code} physical="
        f"{'AnonymousSuperArgs$1 local2 = new AnonymousSuperArgs$1(' in text} "
        f"projected={'new Base(' in text}"
    )


if __name__ == "__main__":
    main()
