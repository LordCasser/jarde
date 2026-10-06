#!/usr/bin/env python3
"""The change's corpus render differential, on two legs.

    loose leg  — every `.class` below `tests/fixtures`, rendered through `single_class` policy
                 (the standalone shape: nothing but the class itself is declared).
    family leg — every directory below `tests/fixtures` that holds class files becomes one jar
                 whose entries are named by each class file's **own internal name** (read from the
                 class file, so packaged and probe layouts both work), and each class in it is
                 rendered through `plain_jar` policy as the root. This is the only leg in which a
                 jar-provided interface definition or a `$`-named parent can be proved.

Both legs are rendered by two binaries: the base (`/tmp/pih-bin/jarde-cli-base`, the tree before
this change) and the patched one. Every render is diffed and the moved classes are printed.

Self-tests before any number is trusted (a harness that cannot fail proves nothing):
  * every text render must carry `// jarde: presentation of`; a render without it is an error
    document (the false-zero source) and is reported as `ERROR-RENDER`, never counted as equal;
  * known positive: `BR$Impl` in the br-family family leg moves from the raw header to
    `implements java.lang.Comparable<BR$Impl>`;
  * known negative: `BareBox` (the `$`-named parent without type arguments) must not move.

    python3 results/05-corpus-sweep.py > results/05-corpus-sweep.out
"""

import os
import shutil
import struct
import subprocess
import sys
import zipfile

BASE = "/tmp/pih-bin/jarde-cli-base"
PATCHED = "/tmp/pih-bin/jarde-cli-patched"
ROOT = "/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a10fff-5ca8-7423-97ff-8582a1fb7e61"
FIXTURES = os.path.join(ROOT, "tests/fixtures")
WORK = "/tmp/pih-sweep-py"


def internal_name(path):
    """The class file's own `this_class` internal name, read from the constant pool.

    A hand-patched probe (this repository commits several by design) can carry a pool this walk
    cannot follow; `None` then means "no name to read", and the caller falls back to the file's
    own base name. Both binaries are asked the same question either way, so a probe the reader
    refuses still compares equal.
    """
    data = open(path, "rb").read()
    count = struct.unpack_from(">H", data, 8)[0]
    pool = {}
    class_entries = {}
    offset = 10
    index = 1
    try:
        while index < count:
            tag = data[offset]
            if tag == 1:
                length = struct.unpack_from(">H", data, offset + 1)[0]
                pool[index] = data[offset + 3 : offset + 3 + length].decode("utf-8", "replace")
                offset += 3 + length
            elif tag == 7:
                class_entries[index] = struct.unpack_from(">H", data, offset + 1)[0]
                offset += 3
            elif tag in (8, 16, 19, 20):
                offset += 3
            elif tag == 15:
                offset += 4
            elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
                offset += 5
            elif tag in (5, 6):
                offset += 9
                index += 1
            else:
                return None
            index += 1
        this_class = struct.unpack_from(">H", data, offset + 2)[0]
        return pool.get(class_entries.get(this_class), None)
    except (struct.error, KeyError, IndexError):
        return None


def class_name(path):
    """The name to render one fixture class under: its own internal name when readable."""
    return internal_name(path) or os.path.basename(path)[: -len(".class")]


def render(binary, policy, jar_or_path, name, out):
    with open(out, "w") as handle:
        subprocess.run(
            [
                binary,
                "class-source",
                "--policy",
                policy,
                "--input",
                jar_or_path,
                "--class",
                name,
                "--format",
                "text",
                "--output",
                out,
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    text = open(out, errors="replace").read()
    return text.startswith("// jarde: presentation of")


def key(path):
    return path.replace(ROOT + "/", "").replace("/", "_")


def main():
    shutil.rmtree(WORK, ignore_errors=True)
    os.makedirs(os.path.join(WORK, "jars"))
    os.makedirs(os.path.join(WORK, "out"))

    classes = []
    for directory, _, names in os.walk(FIXTURES):
        for name in sorted(names):
            if name.endswith(".class"):
                classes.append(os.path.join(directory, name))
    classes.sort()
    print(f"corpus classes: {len(classes)}")

    # ---- family jars (entries named by each class file's own internal name) --------------------
    jars = {}
    directories = sorted({os.path.dirname(path) for path in classes})
    for directory in directories:
        members = [path for path in classes if os.path.dirname(path) == directory]
        jar = os.path.join(WORK, "jars", key(directory) + ".jar")
        with zipfile.ZipFile(jar, "w", zipfile.ZIP_STORED) as archive:
            for path in members:
                archive.write(path, class_name(path) + ".class")
        jars[directory] = jar

    # ---- self-tests ---------------------------------------------------------------------------
    br_dir = os.path.join(FIXTURES, "p3-bridge-projection/br-family/v8")
    br_jar = jars[br_dir]
    base_ok = render(BASE, "plain-jar", br_jar, "BR$Impl", os.path.join(WORK, "out/st-impl-base.txt"))
    base_text = open(os.path.join(WORK, "out/st-impl-base.txt"), errors="replace").read()
    patched_ok = render(
        PATCHED, "plain-jar", br_jar, "BR$Impl", os.path.join(WORK, "out/st-impl-patched.txt")
    )
    patched_text = open(
        os.path.join(WORK, "out/st-impl-patched.txt"), errors="replace"
    ).read()
    if not base_ok or not patched_ok:
        print("SELF-TEST FAILED: a self-test render carries no jarde self-header")
        return 1
    if "implements java.lang.Comparable<BR$Impl>" not in patched_text:
        print("SELF-TEST FAILED: the patched render of BR$Impl does not carry the arguments")
        return 1
    if "implements java.lang.Comparable<BR$Impl>" in base_text:
        print("SELF-TEST FAILED: the base render of BR$Impl already carries the arguments")
        return 1
    parent_dir = os.path.join(FIXTURES, "p3-nested-parent-projection/v8")
    render(BASE, "plain-jar", jars[parent_dir], "BareBox", os.path.join(WORK, "out/st-bare-base.txt"))
    render(
        PATCHED, "plain-jar", jars[parent_dir], "BareBox", os.path.join(WORK, "out/st-bare-patched.txt")
    )
    if open(os.path.join(WORK, "out/st-bare-base.txt"), errors="replace").read() != open(
        os.path.join(WORK, "out/st-bare-patched.txt"), errors="replace"
    ).read():
        print("SELF-TEST FAILED: the bare $ parent moved")
        return 1
    print("SELF-TEST OK: BR$Impl raw -> parameterized; BareBox unchanged")

    # ---- the sweep ----------------------------------------------------------------------------
    moved = []
    errors = []
    for leg in ("loose", "family"):
        for path in classes:
            name = class_name(path)
            if leg == "loose":
                policy, target = "single-class", path
            else:
                policy, target = "plain-jar", jars[os.path.dirname(path)]
            base_out = os.path.join(WORK, "out", f"{leg}-{key(path)}.base.txt")
            patched_out = os.path.join(WORK, "out", f"{leg}-{key(path)}.patched.txt")
            base_ok = render(BASE, policy, target, name, base_out)
            patched_ok = render(PATCHED, policy, target, name, patched_out)
            if not base_ok or not patched_ok:
                errors.append(f"ERROR-RENDER {leg} {path} (base={base_ok} patched={patched_ok})")
                continue
            if open(base_out, errors="replace").read() != open(patched_out, errors="replace").read():
                moved.append(f"MOVED {leg} {path}")
        print(f"{leg} leg: {len(classes)} classes")
    print(f"moved classes: {len(moved)}")
    for line in moved:
        print(line)
    print(f"error renders: {len(errors)}")
    for line in errors:
        print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
