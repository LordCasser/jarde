#!/usr/bin/env python3
"""Rebuild assertion-shape fixtures and verify the patched refusal classfiles."""

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, out, stem):
    result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=60)
    (out / f"{stem}.stdout.txt").write_text(result.stdout, encoding="utf-8")
    (out / f"{stem}.stderr.txt").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"{stem} exited {result.returncode}; see {out / (stem + '.stderr.txt')}")
    return result.stdout


def u2(data, offset):
    return int.from_bytes(data[offset : offset + 2], "big")


def patch_synthetic_field(path, field_name):
    """Add ACC_SYNTHETIC to one javac-built explicit status field; no code bytes change."""
    data = bytearray(path.read_bytes())
    assert data[:4] == bytes.fromhex("ca fe ba be")
    cp_count = u2(data, 8)
    cp = {}
    offset = 10
    index = 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            cp[index] = data[offset : offset + size].decode("utf-8")
            offset += size
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1

    offset += 6  # class access, this_class, super_class
    interface_count = u2(data, offset)
    offset += 2 + 2 * interface_count
    field_count = u2(data, offset)
    offset += 2
    found = []
    for _ in range(field_count):
        flags_at = offset
        flags = u2(data, offset)
        name_index = u2(data, offset + 2)
        descriptor_index = u2(data, offset + 4)
        offset += 6
        attribute_count = u2(data, offset)
        offset += 2
        for _ in range(attribute_count):
            length = int.from_bytes(data[offset + 2 : offset + 6], "big")
            offset += 6 + length
        if cp.get(name_index) == field_name:
            found.append((flags_at, flags, cp.get(descriptor_index)))
    assert len(found) == 1, found
    flags_at, flags, descriptor = found[0]
    assert descriptor == "Z", descriptor
    assert flags & 0x1018 == 0x0018, hex(flags)
    data[flags_at : flags_at + 2] = (flags | 0x1000).to_bytes(2, "big")
    path.write_bytes(data)
    return {"descriptor": descriptor, "flags_before": f"0x{flags:04x}",
            "flags_after": f"0x{flags | 0x1000:04x}", "changed_bytes": 2}


def compile_sources(out, label, sources):
    classes = out / f"{label}-classes"
    classes.mkdir(parents=True, exist_ok=True)
    run(["javac", "--release", "8", "-g", "-Xlint:-options", "-d", str(classes)]
        + [str(source) for source in sources], out, f"{label}-javac")
    return classes


def execute(classes, main, mode, out, label):
    flag = {"enabled": "-ea", "disabled": "-da"}[mode]
    return run(["java", "-Xverify:all", flag, "-cp", str(classes), main],
               out, f"{label}-{mode}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path,
                        help="new or empty directory for generated classes and transcripts")
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        raise RuntimeError(f"output directory must be empty: {out}")
    out.mkdir(parents=True, exist_ok=True)

    sources = {
        "positive": [HERE / "AssertVariants.java", HERE / "AssertVariantsRunner.java"],
        "negative": [HERE / "AssertExtraFieldAccess.java",
                     HERE / "AssertDifferentConstructor.java", HERE / "RejectRunner.java"],
    }
    javac_version = run(["javac", "-version"], out, "javac-version").strip()
    run(["java", "-version"], out, "java-version")
    java_version = (out / "java-version.stderr.txt").read_text(encoding="utf-8").splitlines()[0]
    jadx_version = run(["jadx", "--version"], out, "jadx-version").strip()
    summary = {"tools": {"javac": javac_version,
                         "java": java_version,
                         "jadx": jadx_version,
                         "jarde_cli": "unavailable: target/debug/jarde-cli is absent; no build was started"},
               "sources": {path.name: sha(path) for group in sources.values() for path in group}}

    positive = compile_sources(out, "positive", sources["positive"])
    positive_class = positive / "AssertVariants.class"
    summary["positive"] = {"class_sha256": sha(positive_class), "modes": {}}
    javap = run(["javap", "-p", "-c", "-v", str(positive_class)], out, "javap-positive")
    summary["positive"]["assert_bcis"] = [line.strip() for line in javap.splitlines()
                                           if "AssertionError" in line or "desiredAssertionStatus" in line
                                           or "assertionsDisabled" in line]
    for mode in ("enabled", "disabled"):
        stdout = execute(positive, "AssertVariantsRunner", mode, out, f"positive-{mode}")
        summary["positive"]["modes"][mode] = stdout
    assert summary["positive"]["modes"] == {
        "enabled": "init=1;plain-true=12;plain-false=AssertionError:null;multiple-true=12222;"
                   "multiple-second-false=AssertionError:message;effects=12222223\n",
        "disabled": "init=1;plain-true=1;plain-false=none;multiple-true=1;"
                    "multiple-second-false=none;effects=1\n",
    }

    positive_jadx = out / "positive-jadx"
    run(["jadx", "--no-res", "-d", str(positive_jadx), str(positive_class)], out,
        "positive-jadx")
    positive_jadx_source = positive_jadx / "sources/defpackage/AssertVariants.java"
    assert positive_jadx_source.is_file()
    positive_runner_dir = out / "positive-jadx-overlay"
    positive_runner_dir.mkdir()
    positive_jadx_runner = positive_runner_dir / "AssertVariantsRunner.java"
    positive_jadx_runner.write_text(
        "package defpackage;\n" + sources["positive"][1].read_text(encoding="utf-8"),
        encoding="utf-8")
    positive_jadx_classes = compile_sources(out, "positive-jadx", [
        positive_jadx_source, positive_jadx_runner])
    summary["positive"]["jadx_source_sha256"] = sha(positive_jadx_source)
    summary["positive"]["jadx_modes"] = {
        mode: execute(positive_jadx_classes, "defpackage.AssertVariantsRunner", mode, out,
                      f"positive-jadx-{mode}") for mode in ("enabled", "disabled")}
    assert summary["positive"]["jadx_modes"] == summary["positive"]["modes"]

    negative = compile_sources(out, "negative", sources["negative"])
    patched = {}
    for name in ("AssertExtraFieldAccess", "AssertDifferentConstructor"):
        class_file = negative / f"{name}.class"
        field_name = "$assertionsDisabled"
        patched[name] = {"patch": patch_synthetic_field(class_file, field_name),
                         "class_sha256": sha(class_file)}
        javap = run(["javap", "-p", "-c", "-v", str(class_file)], out, f"javap-{name}")
        patched[name]["interesting_javap"] = [line.strip() for line in javap.splitlines()
                                               if field_name in line or "AssertionError" in line
                                               or "desiredAssertionStatus" in line]
    summary["negative"] = patched
    for mode in ("enabled", "disabled"):
        summary.setdefault("negative_runner", {})[mode] = execute(
            negative, "RejectRunner", mode, out, f"negative-{mode}")

    expected = {
        "enabled": "extra=AssertionError:null:writes=1;probe=false;ctor=AssertionError:42;\n",
        "disabled": "probe=true;\n",
    }
    assert summary["negative_runner"] == expected, summary["negative_runner"]
    # Loading and invoking both classes under -Xverify:all above verifies each patched file.
    negative_jadx = out / "negative-jadx"
    run(["jadx", "--no-res", "-d", str(negative_jadx),
         str(negative / "AssertExtraFieldAccess.class"),
         str(negative / "AssertDifferentConstructor.class")], out, "negative-jadx")
    negative_jadx_sources = [
        negative_jadx / "sources/defpackage/AssertExtraFieldAccess.java",
        negative_jadx / "sources/defpackage/AssertDifferentConstructor.java",
    ]
    assert all(path.is_file() for path in negative_jadx_sources)
    negative_runner_dir = out / "negative-jadx-overlay"
    negative_runner_dir.mkdir()
    negative_jadx_runner = negative_runner_dir / "RejectRunner.java"
    negative_jadx_runner.write_text(
        "package defpackage;\n" + sources["negative"][2].read_text(encoding="utf-8"),
        encoding="utf-8")
    negative_jadx_classes = compile_sources(out, "negative-jadx",
                                            negative_jadx_sources + [negative_jadx_runner])
    summary["negative"]["jadx_source_sha256"] = {
        path.name: sha(path) for path in negative_jadx_sources}
    summary["negative"]["jadx_modes"] = {
        mode: execute(negative_jadx_classes, "defpackage.RejectRunner", mode, out,
                      f"negative-jadx-{mode}") for mode in ("enabled", "disabled")}
    assert summary["negative"]["jadx_modes"] == summary["negative_runner"]
    (out / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
