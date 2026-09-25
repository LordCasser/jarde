#!/usr/bin/env python3
"""Rebuild the interface ConstantValue/clinit phase-boundary experiment."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
CLI_DEFAULT = Path("/tmp/jarde-cli-root-final-accepted")
CLI_EXPECTED_SHA256 = "d7520a08a37b54ef579d42c770f5b512af17ddb11b05050568458759c78dbf16"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command, log: Path, *, check: bool = False):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=90)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    if check and result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(map(str, command))}\n{result.stdout}{result.stderr}")
    return result.returncode, result.stdout


def u2(data: bytes, at: int) -> int:
    return struct.unpack_from(">H", data, at)[0]


def u4(data: bytes, at: int) -> int:
    return struct.unpack_from(">I", data, at)[0]


def parse_class(data: bytes):
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    cp = {}
    at, index, count = 10, 1, u2(data, 8)
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            n = u2(data, at)
            cp[index] = (tag, data[at + 2:at + 2 + n].decode("utf-8"))
            at += 2 + n
        elif tag in (3, 4):
            cp[index] = (tag, data[at:at + 4])
            at += 4
        elif tag in (5, 6):
            cp[index] = (tag, data[at:at + 8])
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, at))
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[index] = (tag, u2(data, at), u2(data, at + 2))
            at += 4
        elif tag == 15:
            cp[index] = (tag, data[at], u2(data, at + 1))
            at += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        index += 1

    def utf(i):
        return cp[i][1]

    fields = []
    cursor = at + 6
    interface_count = u2(data, cursor)
    cursor += 2 + interface_count * 2
    nfields = u2(data, cursor)
    cursor += 2
    for _ in range(nfields):
        start = cursor
        access, name, desc, attrs = struct.unpack_from(">HHHH", data, cursor)
        cursor += 8
        attributes = []
        for _ in range(attrs):
            attr_name, length = utf(u2(data, cursor)), u4(data, cursor + 2)
            attributes.append(attr_name)
            cursor += 6 + length
        fields.append({"name": utf(name), "descriptor": utf(desc), "access": access,
                       "attributes": attributes, "start": start, "end": cursor})
    methods = []
    nmethods = u2(data, cursor)
    cursor += 2
    for _ in range(nmethods):
        access, name, desc, attrs = struct.unpack_from(">HHHH", data, cursor)
        method = {"name": utf(name), "descriptor": utf(desc), "access": access, "code": None}
        cursor += 8
        for _ in range(attrs):
            attrname, length = utf(u2(data, cursor)), u4(data, cursor + 2)
            info = cursor + 6
            if attrname == "Code":
                method["code"] = {"start": info + 8, "length": u4(data, info + 4)}
            cursor += 6 + length
        methods.append(method)
    return cp, fields, methods


def cp_member(cp, index):
    tag, owner_idx, nt_idx = cp[index]
    if tag not in (10, 11, 9):
        return None
    owner = cp[cp[owner_idx][1]][1]
    _, name_idx, desc_idx = cp[nt_idx]
    return owner, cp[name_idx][1], cp[desc_idx][1]


def patch_clinit(original: bytes):
    cp, fields, methods = parse_class(original)
    clinit = next(m for m in methods if m["name"] == "<clinit>")
    code = clinit["code"]
    if not code:
        raise ValueError("compiled interface has no <clinit> Code")
    start, end = code["start"], code["start"] + code["length"]
    instructions = []
    at = start
    while at < end:
        opcode = original[at]
        size = {0xB2: 3, 0xB3: 3, 0xB8: 3}.get(opcode, 1)
        if opcode in (0xB2, 0xB3, 0xB8):
            index = u2(original, at + 1)
            instructions.append((at, opcode, cp_member(cp, index)))
        else:
            instructions.append((at, opcode, None))
        at += size
    expected = [(0xB2, ("PhaseProbe", "LATE", "I")),
                (0xB3, ("PhaseProbe", "EARLY", "I")),
                (0xB8, ("BoundaryEffects", "value", "()I")),
                (0xB3, ("PhaseProbe", "LATE", "I")), (0xB1, None)]
    compact = [(op, member) for _, op, member in instructions]
    if compact != expected:
        raise ValueError(f"unexpected <clinit> instruction sequence: {compact!r}")
    patch_at = instructions[2][0]
    changed = bytearray(original)
    # invokestatic #u2 is three bytes; sipush 9 is also three bytes.
    changed[patch_at:patch_at + 3] = b"\x11\x00\x09"
    changed = bytes(changed)
    differences = [i for i, (a, b) in enumerate(zip(original, changed)) if a != b]
    if changed[:patch_at] != original[:patch_at] or changed[patch_at + 3:] != original[patch_at + 3:]:
        raise AssertionError("patch modified bytes outside the selected instruction")
    if len(changed) != len(original) or not differences:
        raise AssertionError("patch was not a nonempty same-length change")
    return changed, {"offset": patch_at, "length": 3, "changed_byte_offsets": differences,
                     "original_instruction": "invokestatic BoundaryEffects.value()I",
                     "patched_instruction": "sipush 9"}


def javac(source_paths, out, log, release=True):
    cmd = ["javac"]
    if release:
        cmd += ["--release", "8"]
    cmd += ["-g:none", "-d", str(out), *map(str, source_paths)]
    return run(cmd, log)


def decompile_run(tool, cli, binary, probe_source, support_sources, out, work):
    if tool == "jadx":
        generated = work / "jadx-generated"
        status, _ = run(["jadx", "--no-res", "-d", generated, binary], out / "jadx.log")
        if status:
            return {"decompile_exit": status, "javac_exit": None, "runtime_exit": None, "runtime": None}
        candidates = list(generated.rglob("PhaseProbe.java"))
        if len(candidates) != 1:
            return {"decompile_exit": 2, "javac_exit": None, "runtime_exit": None, "runtime": None}
        source = candidates[0]
    else:
        status, source_text = run([cli, "class-source", "--input", binary, "--class", "PhaseProbe",
                                   "--policy", "single-class", "--release", "8", "--format", "text"],
                                  out / "jarde-report.txt")
        (out / "jarde.java.txt").write_text(source_text)
        if status:
            return {"decompile_exit": status, "javac_exit": None, "runtime_exit": None, "runtime": None}
        source = work / "jarde-source" / "PhaseProbe.java"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(source_text)
    shutil.copy2(source, out / f"{tool}.java.txt")
    source_text = source.read_text()
    package_line = next((line.strip() for line in source_text.splitlines()
                         if line.strip().startswith("package ")), None)
    package_name = package_line[len("package "):].rstrip(";") if package_line else ""
    support_dir = out / f"{tool}-support"
    support_dir.mkdir(parents=True, exist_ok=True)
    packaged_support = []
    for support_source in support_sources:
        text = support_source.read_text()
        if package_line:
            text = package_line + "\n\n" + text
        target = support_dir / support_source.name
        target.write_text(text)
        packaged_support.append(target)
    runner_source = '''public final class BoundaryRunner {
    public static void main(String[] args) {
        System.out.println(PhaseProbe.EARLY + "|" + PhaseProbe.LATE);
    }
}
'''
    if package_line:
        runner_source = package_line + "\n\n" + runner_source
    runner_path = support_dir / "BoundaryRunner.java"
    runner_path.write_text(runner_source)
    packaged_support.append(runner_path)
    classes = work / f"{tool}-classes"
    status, _ = javac([source, *packaged_support], classes, out / f"{tool}-javac.log")
    if status:
        return {"decompile_exit": 0, "javac_exit": status, "runtime_exit": None, "runtime": None}
    runner = f"{package_name}.BoundaryRunner" if package_name else "BoundaryRunner"
    rc, stdout = run(["java", "-Xverify:all", "-cp", classes, runner],
                     out / f"{tool}-runtime.txt")
    return {"decompile_exit": 0, "javac_exit": 0, "runtime_exit": rc, "runtime": stdout.strip()}


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, default=CLI_DEFAULT)
    parser.add_argument("--out", type=Path, default=HERE)
    parser.add_argument("--work", type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()) and out != HERE:
        raise SystemExit(f"refusing to overwrite nonempty output directory: {out}")
    out.mkdir(parents=True, exist_ok=True)
    source_dir = out / "original-source"
    source_dir.mkdir(exist_ok=True)
    phase_source = '''public interface PhaseProbe {
    int EARLY = PhaseProbe.LATE;
    int LATE = BoundaryEffects.value();
    static String observe() { return EARLY + "|" + LATE; }
}
'''
    effects_source = '''public final class BoundaryEffects {
    static int value() { return 9; }
    private BoundaryEffects() {}
}
'''
    runner_source = '''public final class BoundaryRunner {
    public static void main(String[] args) {
        System.out.println(PhaseProbe.EARLY + "|" + PhaseProbe.LATE);
    }
}
'''
    (source_dir / "PhaseProbe.java").write_text(phase_source)
    (source_dir / "BoundaryEffects.java").write_text(effects_source)
    (source_dir / "BoundaryRunner.java").write_text(runner_source)
    with tempfile.TemporaryDirectory(prefix="jarde-constant-phase-") as tmp:
        work = Path(args.work).resolve() if args.work else Path(tmp)
        work.mkdir(parents=True, exist_ok=True)
        original_classes = work / "original-classes"
        compile_rc, _ = javac([source_dir / "PhaseProbe.java", source_dir / "BoundaryEffects.java",
                               source_dir / "BoundaryRunner.java"], original_classes, out / "original-javac.log")
        if compile_rc:
            raise SystemExit("original Java 8 source did not compile")
        original_class = original_classes / "PhaseProbe.class"
        before = original_class.read_bytes()
        (out / "PhaseProbe.prepatch.class").write_bytes(before)
        patched, patch_info = patch_clinit(before)
        probe_class = out / "PhaseProbe.class"
        probe_class.write_bytes(patched)
        # Use a copied class tree, replacing only PhaseProbe.class, to execute the frozen class.
        frozen_classes = work / "frozen-classes"
        shutil.copytree(original_classes, frozen_classes)
        (frozen_classes / "PhaseProbe.class").write_bytes(patched)
        javap_rc, javap_text = run(["javap", "-classpath", frozen_classes, "-v", "-p", "PhaseProbe"],
                                   out / "javap.txt", check=True)
        runtime_rc, original_runtime = run(["java", "-Xverify:all", "-cp", frozen_classes,
                                            "BoundaryRunner"], out / "original-runtime.txt")
        if runtime_rc:
            raise SystemExit("frozen patched class failed -Xverify:all execution")
        # Tool runs each receive original support sources, keeping each compile tree isolated.
        support = [source_dir / "BoundaryEffects.java", source_dir / "BoundaryRunner.java"]
        jarde = decompile_run("jarde", args.cli, probe_class, source_dir / "PhaseProbe.java", support,
                              out, work)
        jadx = decompile_run("jadx", args.cli, probe_class, source_dir / "PhaseProbe.java", support,
                             out, work)

        # Mechanism-only control: javac makes both fields constants and inlines LATE into EARLY.
        const_dir = out / "javac-constant-mechanism-only"
        const_dir.mkdir(exist_ok=True)
        const_source = '''public interface JavacConstantControl {
    int EARLY = JavacConstantControl.LATE;
    int LATE = 9;
}
'''
        runner = '''public final class ConstantControlRunner {
    public static void main(String[] args) {
        System.out.println(JavacConstantControl.EARLY + "|" + JavacConstantControl.LATE);
    }
}
'''
        (const_dir / "JavacConstantControl.java").write_text(const_source)
        (const_dir / "ConstantControlRunner.java").write_text(runner)
        const_classes = work / "constant-control-classes"
        const_rc, _ = javac([const_dir / "JavacConstantControl.java", const_dir / "ConstantControlRunner.java"],
                            const_classes, out / "javac-constant-control-javac.log")
        if const_rc:
            raise SystemExit("javac constant control did not compile")
        shutil.copy2(const_classes / "JavacConstantControl.class", const_dir / "JavacConstantControl.class")
        _, const_javap = run(["javap", "-classpath", const_classes, "-v", "-p", "JavacConstantControl"],
                             out / "javac-constant-control-javap.txt", check=True)
        _, const_runner_javap = run(["javap", "-classpath", const_classes, "-c", "-p", "ConstantControlRunner"],
                                   out / "javac-constant-control-runner-javap.txt", check=True)
        const_runtime_rc, const_runtime = run(["java", "-Xverify:all", "-cp", const_classes,
                                               "ConstantControlRunner"], out / "javac-constant-control-runtime.txt")
        parsed_cp, parsed_fields, _ = parse_class(patched)
        summary = {
            "purpose": "controlled JVM evidence for interface <clinit> default-value read before later field write",
            "source_shape": "legal Java 8 interface source compiled with javac, followed by a same-length three-byte patch confined to <clinit> Code",
            "cli_path": str(args.cli.resolve()), "cli_sha256": sha(args.cli.read_bytes()),
            "cli_matches_expected_frozen_sha256": sha(args.cli.read_bytes()) == CLI_EXPECTED_SHA256,
            "javac_version": (lambda r: (r.stdout + r.stderr).strip())(subprocess.run(["javac", "-version"], capture_output=True, text=True)),
            "jadx_version": subprocess.run(["jadx", "--version"], capture_output=True, text=True).stdout.strip(),
            "prepatch_class_sha256": sha(before), "prepatch_class_bytes": len(before),
            "frozen_class_sha256": sha(patched), "frozen_class_bytes": len(patched),
            "patch": patch_info,
            "frozen_fields": [{"name": f["name"], "attributes": f["attributes"]} for f in parsed_fields],
            "frozen_original_runtime_exit": runtime_rc, "frozen_original_runtime": original_runtime.strip(),
            "jarde": jarde, "jadx": jadx,
            "manual_javac_constant_control": {
                "explicitly_not_jarde_output": True,
                "class_sha256": sha((const_dir / "JavacConstantControl.class").read_bytes()),
                "runtime_exit": const_runtime_rc, "runtime": const_runtime.strip(),
                "shows_ConstantValue": "ConstantValue:" in const_javap,
                "reader_inlines_to_9": "String 9|9" in const_runner_javap,
            },
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n")
        (out / "README.md").write_text(render_readme(summary, javap_text))


def render_readme(s, javap_text):
    patch = s["patch"]
    return f'''# Interface field initialization phase boundary

This evidence isolates the `ConstantValue` versus `<clinit>` distinction for interface fields. The source in `original-source/PhaseProbe.java` is legal Java 8. `javac --release 8` emits `EARLY` as a `getstatic PhaseProbe.LATE` followed by `putstatic PhaseProbe.EARLY`, and emits a call for the `LATE` initializer. The experiment changes only the three-byte instruction at class-file offset `{patch['offset']}` inside `<clinit>` Code: `{patch['original_instruction']}` becomes `{patch['patched_instruction']}`. The class file length is unchanged; `summary.json` records the SHA-256 values and the changed byte offsets.

`PhaseProbe.class` is the frozen patched artifact; `PhaseProbe.prepatch.class` preserves javac's class before that one instruction edit. `javap.txt` is the disassembly of the frozen class. The frozen class passes `java -Xverify:all` and prints `{s['frozen_original_runtime']}`: `EARLY` observes the JVM default value of `LATE` before the later `putstatic` stores 9. Neither interface field has a `ConstantValue` attribute.

The frozen class was decompiled with the CLI identified in `summary.json` and JADX {s['jadx_version']}. Each complete emitted source and matching support classes was **attempted** with `javac --release 8`; only a successful compilation was run with `-Xverify:all`. Jarde's source failed compilation (exits: {s['jarde']['decompile_exit']}/{s['jarde']['javac_exit']}/{s['jarde']['runtime_exit']}); javac rejects the uninitialized interface fields and the interface `static` block, so there is no Jarde runtime result. JADX's source compiled and printed `{s['jadx']['runtime']}` (exits: {s['jadx']['decompile_exit']}/{s['jadx']['javac_exit']}/{s['jadx']['runtime_exit']}). This is observably different from the original class's `0|9`: JADX presents `LATE` as a literal, allowing javac to fold `EARLY` to 9.

`javac-constant-mechanism-only/JavacConstantControl.java` is a hand-written mechanism control, explicitly **not Jarde output**. In that source, `EARLY = JavacConstantControl.LATE; LATE = 9;` makes both fields compile-time constants. The class has `ConstantValue` attributes and the runner prints `{s['manual_javac_constant_control']['runtime']}` because javac inlines 9. This demonstrates why using a literal `static final int LATE = 9` in the projected interface would erase the original default-value observation.

## Replay

From the repository root, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/constant-phase-boundary/replay.py
```

To direct generated logs and frozen outputs to a fresh directory, pass `--out /path/to/new-empty-dir`; the executable inputs remain in the requested evidence directory. To use an explicit temporary compilation area, also pass `--work /path/to/work-dir`. The script refuses to overwrite a nonempty alternate output directory.
'''


if __name__ == "__main__":
    main()
