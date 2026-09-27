#!/usr/bin/env python3
"""Rebuild and challenge the finite DT-29 receiver-binding certificate."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixtures"
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
FIXED_SOURCE_SHA = "daadc9da3711519ed2d0c7f2157c420d2a13540ed6898f182a3f64591be0c2af"
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JARDE = Path(os.environ["JARDE_CLI"])
EXPECTED = "101:true:false:true:false\n"


def run(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def require(test: bool, message: str) -> None:
    if not test:
        raise AssertionError(message)


def compile_run(sources: list[Path], runner: Path, out: Path) -> dict[str, object]:
    out.mkdir()
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US",
                   "--release", "8", "-g:none", "-Xlint:-options", "-d", out,
                   *sources, runner)
    require(compiled.returncode == 0, compiled.stderr)
    result = run("java", "-Xverify:all", "-cp", out,
                 "dt29p1.ReceiverBindingRunner")
    require(result.returncode == 0 and result.stdout == EXPECTED and result.stderr == "",
            f"verification/run mismatch: {result.stdout!r} {result.stderr!r}")
    return {"compile_exit": compiled.returncode, "run_exit": result.returncode,
            "stdout": result.stdout}


def jar_of(classes: Path, jar: Path) -> list[str]:
    names = []
    with ZipFile(jar, "w", ZIP_DEFLATED) as archive:
        for path in sorted(classes.rglob("*.class")):
            if path.name == "ReceiverBindingRunner.class":
                continue
            name = path.relative_to(classes).as_posix()
            names.append(name.removesuffix(".class"))
            entry = ZipInfo(name, date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            archive.writestr(entry, path.read_bytes())
    return names


def cp(data: bytearray) -> tuple[dict[int, dict[str, object]], int]:
    pool: dict[int, dict[str, object]] = {}
    count = int.from_bytes(data[8:10], "big")
    at = 10
    index = 1
    while index < count:
        start = at
        tag = data[at]
        at += 1
        entry: dict[str, object] = {"tag": tag, "start": start}
        if tag == 1:
            length = int.from_bytes(data[at:at + 2], "big")
            entry["text"] = bytes(data[at + 2:at + 2 + length]).decode("utf-8")
            at += 2 + length
        elif tag in (3, 4):
            at += 4
        elif tag in (5, 6):
            at += 8
            pool[index] = entry
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entry["index"] = int.from_bytes(data[at:at + 2], "big")
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            entry["first"] = int.from_bytes(data[at:at + 2], "big")
            entry["second"] = int.from_bytes(data[at + 2:at + 4], "big")
            at += 4
        elif tag == 15:
            at += 3
        else:
            raise AssertionError(f"unexpected constant pool tag {tag}")
        pool[index] = entry
        index += 1
    return pool, at


def text(pool: dict[int, dict[str, object]], index: int) -> str:
    entry = pool[index]
    return text(pool, int(entry["index"])) if entry["tag"] == 7 else str(entry["text"])


def change_fieldref(data: bytes, *, owner: str | None = None,
                    name: str | None = None, descriptor: str | None = None) -> bytes:
    result = bytearray(data)
    pool, _ = cp(result)
    entry = next(item for item in pool.values() if item["tag"] == 9
                 and text(pool, int(pool[int(item["second"])]["first"])) == "protectedField")
    nat = pool[int(entry["second"])]
    if owner is not None:
        chosen = next(index for index, item in pool.items()
                      if item["tag"] == 7 and text(pool, index) == owner)
        start = int(entry["start"]) + 1
    elif name is not None:
        chosen = next(index for index, item in pool.items()
                      if item["tag"] == 1 and item["text"] == name)
        start = int(nat["start"]) + 1
    else:
        candidates = [index for index, item in pool.items()
                      if item["tag"] == 1 and item["text"] == descriptor]
        if not candidates:
            current = pool[int(nat["second"])]
            require(current["text"] == "Z" and descriptor == "I",
                    "only one-byte Z-to-I descriptor mutation is supported")
            result[int(current["start"]) + 3] = ord("I")
            return bytes(result)
        chosen = candidates[0]
        start = int(nat["start"]) + 3
    result[start:start + 2] = chosen.to_bytes(2, "big")
    return bytes(result)


def change_bits_owner(data: bytes) -> bytes:
    result = bytearray(data)
    pool, _ = cp(result)
    entry = next(item for item in pool.values() if item["tag"] == 10
                 and text(pool, int(pool[int(item["second"])]["first"])) == "bits")
    chosen = next(index for index, item in pool.items() if item["tag"] == 7
                  and text(pool, index) == "dt29p1/ReceiverBindingFamily$C")
    result[int(entry["start"]) + 1:int(entry["start"]) + 3] = chosen.to_bytes(2, "big")
    return bytes(result)


def change_bits_name(data: bytes) -> bytes:
    result = bytearray(data)
    pool, _ = cp(result)
    entry = next(item for item in pool.values() if item["tag"] == 10
                 and text(pool, int(pool[int(item["second"])]["first"])) == "bits")
    nat = pool[int(entry["second"])]
    chosen = next(index for index, item in pool.items()
                  if item["tag"] == 1 and item["text"] == "run")
    result[int(nat["start"]) + 1:int(nat["start"]) + 3] = chosen.to_bytes(2, "big")
    return bytes(result)


def change_parent(data: bytes) -> bytes:
    result = bytearray(data)
    pool, at = cp(result)
    self_index = int.from_bytes(result[at + 2:at + 4], "big")
    result[at + 4:at + 6] = self_index.to_bytes(2, "big")
    return bytes(result)


def change_field_flags(data: bytes, flags: int) -> bytes:
    result = bytearray(data)
    pool, at = cp(result)
    at += 6
    count = int.from_bytes(result[at:at + 2], "big")
    at += 2 + 2 * count
    count = int.from_bytes(result[at:at + 2], "big")
    at += 2
    for _ in range(count):
        start = at
        name = text(pool, int.from_bytes(result[at + 2:at + 4], "big"))
        attrs = int.from_bytes(result[at + 6:at + 8], "big")
        at += 8
        for _ in range(attrs):
            at += 6 + int.from_bytes(result[at + 2:at + 6], "big")
        if name == "protectedField":
            result[start:start + 2] = flags.to_bytes(2, "big")
            return bytes(result)
    raise AssertionError("protectedField declaration absent")


def method_attribute(data: bytearray, wanted: str) -> tuple[dict[int, dict[str, object]], int, int]:
    pool, at = cp(data)
    at += 6
    interfaces = int.from_bytes(data[at:at + 2], "big")
    at += 2 + interfaces * 2
    fields = int.from_bytes(data[at:at + 2], "big")
    at += 2
    for _ in range(fields):
        attrs = int.from_bytes(data[at + 6:at + 8], "big")
        at += 8
        for _ in range(attrs):
            at += 6 + int.from_bytes(data[at + 2:at + 6], "big")
    methods = int.from_bytes(data[at:at + 2], "big")
    at += 2
    for _ in range(methods):
        start = at
        name = text(pool, int.from_bytes(data[at + 2:at + 4], "big"))
        attrs = int.from_bytes(data[at + 6:at + 8], "big")
        at += 8
        if name == wanted:
            return pool, start, at
        for _ in range(attrs):
            at += 6 + int.from_bytes(data[at + 2:at + 6], "big")
    raise AssertionError(f"method {wanted} absent")


def change_ssa_receiver(data: bytes) -> bytes:
    result = bytearray(data)
    pool, _, at = method_attribute(result, "set")
    attrs = int.from_bytes(result[at - 2:at], "big")
    for _ in range(attrs):
        name = text(pool, int.from_bytes(result[at:at + 2], "big"))
        length = int.from_bytes(result[at + 2:at + 6], "big")
        if name == "Code":
            code = at + 6 + 8
            require(result[code + 5] == 0x2b, "BCI 5 is not aload_1")
            result[code + 5] = 0x2a  # the BCI 7 receiver is now C's `this`
            return bytes(result)
        at += 6 + length
    raise AssertionError("Code attribute absent")


def change_method_name(data: bytes, old: str, new: str) -> bytes:
    result = bytearray(data)
    pool, at, _ = method_attribute(result, old)
    new_index = next(index for index, item in pool.items()
                     if item["tag"] == 1 and item["text"] == new)
    result[at + 2:at + 4] = new_index.to_bytes(2, "big")
    return bytes(result)


def change_method_descriptor(data: bytes, name: str, new: str) -> bytes:
    result = bytearray(data)
    pool, at, _ = method_attribute(result, name)
    chosen = next(index for index, item in pool.items()
                  if item["tag"] == 1 and item["text"] == new)
    result[at + 4:at + 6] = chosen.to_bytes(2, "big")
    return bytes(result)


def change_class_package(data: bytes) -> bytes:
    result = bytearray(data)
    pool, _ = cp(result)
    entry = next(item for item in pool.values() if item["tag"] == 1
                 and item["text"] == "dt29p1/ReceiverBindingFamily$C")
    result[int(entry["start"]) + 3] = ord("x")
    return bytes(result)


def variant(work: Path, label: str, base: dict[str, bytes],
            overrides: dict[str, bytes]) -> Path:
    path = work / f"{label}.jar"
    with ZipFile(path, "w", ZIP_DEFLATED) as archive:
        for name, data in sorted((base | overrides).items()):
            archive.writestr(name, data)
    return path


def report(jar: Path, class_name: str) -> dict[str, object]:
    result = run(JARDE, "class-source", "--input", jar, "--class", class_name,
                 "--policy", "plain-jar", "--release", "8", "--format", "json",
                 "--evidence", "all")
    require(result.returncode in (0, 4), result.stderr[-1000:])
    return json.loads(result.stdout)


def body(jar: Path, class_name: str, name: str) -> dict[str, object]:
    item = next(method for method in report(jar, class_name)["methods"]
                if method["item"]["identity"]["name"] == list(name.encode()))
    return item["outcome"]["report"]


def mapped_sites(method: dict[str, object], bcis: list[int]) -> dict[str, list[dict[str, object]]]:
    def compact(origin: dict[str, object]) -> dict[str, object]:
        primary = origin["primary"]
        identity = primary["method"]
        owner = identity["owner"]
        return {
            "primary_bci": primary["bci"],
            "cp": primary["cp"],
            "method_name": bytes(identity["name"]).decode(),
            "method_descriptor": bytes(identity["descriptor"]).decode(),
            "owner_class": bytes(owner["location"]["entry"]["raw_name"]).decode(),
            "owner_sha256": owner["class_bytes"]["digest"],
            "derived_bcis": [item["bci"] for item in origin["derived"]],
        }

    mapped = {}
    for bci in bcis:
        mapped[str(bci)] = [{"text": method["text"][segment["start"]:segment["end"]],
                             "origin": compact(segment["origin"])}
                            for segment in method["source_map"]["segments"]
                            if segment["origin"]["primary"]["bci"] == bci]
        require(mapped[str(bci)], f"BCI {bci} missing from source map")
    return mapped


def fixed_combined(work: Path) -> dict[str, object]:
    source = HERE.parents[1] / "dt29-reference-cast-audit" / "combined" / "inputs" / "FieldCast.java"
    require(source.is_file(), "fixed combined FieldCast source missing")
    require(hashlib.sha256(source.read_bytes()).hexdigest() == FIXED_SOURCE_SHA,
            "fixed combined FieldCast source hash changed")
    classes = work / "fixed-classes"
    classes.mkdir()
    compiled = run("javac", "--release", "8", "-g:none", "-Xlint:-options",
                   "-d", classes, source)
    require(compiled.returncode == 0, compiled.stderr)
    jar = work / "fixed.jar"
    jar_of(classes, jar)
    methods = {}
    source_maps = {}
    for short in ("C", "D"):
        method = body(jar, f"dt29/FieldCast${short}", "set")
        bcis = [2, 7, 12, 17]
        require("@bytecode" not in method["text"], f"fixed {short}.set still falls back")
        require(all(any(segment["origin"]["primary"]["bci"] == bci for segment in
                        method["source_map"]["segments"]) for bci in bcis),
                f"fixed {short}.set lost a physical BCI")
        require("FieldCast$A.access$002((dt29.FieldCast$A) arg1, arg2)" in method["text"],
                f"fixed {short}.set lost physical accessor")
        physical = run("javap", "-classpath", jar, "-c", "-p", f"dt29.FieldCast${short}")
        require(physical.returncode == 0 and
                all(f"Field dt29/FieldCast$A.{name}:Z" in physical.stdout for name in
                    ("publicField", "protectedField", "packagePrivateField")) and
                "Method dt29/FieldCast$A.access$002:(Ldt29/FieldCast$A;Z)Z" in physical.stdout,
                f"fixed {short}.set CP member identity changed")
        methods[short + ".set"] = bcis
        source_maps[short + ".set"] = mapped_sites(method, bcis)
    root = body(jar, "dt29/FieldCast", "run")
    calls = [14, 31, 74]
    require(root["text"].count("bits((dt29.FieldCast$A)") == 3,
            "fixed run lost private bits(A) binding")
    require(all(any(segment["origin"]["primary"]["bci"] == bci and
                    "bits((dt29.FieldCast$A)" in root["text"][segment["start"]:segment["end"]]
                    for segment in root["source_map"]["segments"]) for bci in calls),
            "fixed run lost a private call source BCI")
    methods["run.bits(A)"] = calls
    source_maps["run.bits(A)"] = mapped_sites(root, calls)
    return {"source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "methods": methods, "source_maps": source_maps,
            "physical_field_owner": "dt29/FieldCast$A",
            "physical_accessor": "dt29/FieldCast$A.access$002:(Ldt29/FieldCast$A;Z)Z",
            "physical_private_target": "dt29/FieldCast.bits:(Ldt29/FieldCast$A;)Ljava/lang/String;"}


def main() -> None:
    require(JARDE.is_file(), "JARDE_CLI must point to a built jarde-cli")
    require(run("git", "-C", JADX_ROOT, "rev-parse", "HEAD").stdout.strip() == JADX_HEAD,
            "fixed JADX checkout changed")
    require(not run("git", "-C", JADX_ROOT, "status", "--porcelain").stdout.strip(),
            "fixed JADX checkout has local changes")
    require(JADX.is_file(), "fixed JADX executable missing")
    runner = FIXTURE / "ReceiverBindingRunner.java"
    source = FIXTURE / "ReceiverBindingFamily.java"
    with tempfile.TemporaryDirectory(prefix="jarde-dt29-p1-") as temporary:
        work = Path(temporary)
        original = compile_run([source], runner, work / "original")
        jar = work / "input.jar"
        names = jar_of(work / "original", jar)
        require(len(names) == 5, f"expected five physical classes: {names}")
        jadx_dir = work / "jadx"
        jadx = run(JADX, "-d", jadx_dir, jar)
        require(jadx.returncode == 0, jadx.stderr)
        jadx_result = compile_run(sorted(jadx_dir.rglob("*.java")), runner, work / "jadx-classes")
        jarde_dir = work / "jarde"
        for name in names:
            generated = run(JARDE, "class-source", "--input", jar, "--class", name,
                            "--policy", "plain-jar", "--release", "8", "--format", "text")
            require(generated.returncode == 0 and "@bytecode" not in generated.stdout,
                    f"{name} did not recover: {generated.stdout[-2000:]}")
            path = jarde_dir / (name + ".java")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(generated.stdout)
        jarde_result = compile_run(sorted(jarde_dir.rglob("*.java")), runner, work / "jarde-classes")

        source_bcis = {}
        source_maps = {}
        for class_name in ("ReceiverBindingFamily$C", "ReceiverBindingFamily$D"):
            method = body(jar, "dt29p1/" + class_name, "set")
            text_body = method["text"]
            for bci, member in ((2, "publicField"), (7, "protectedField"),
                                (12, "packageField")):
                require(f"((dt29p1.ReceiverBindingFamily$A) arg1).{member} = arg2;" in text_body,
                        f"{class_name} BCI {bci} lost owner cast")
                require(any(segment["origin"]["primary"]["bci"] == bci for segment in
                            method["source_map"]["segments"]), f"{class_name} BCI {bci} lost source")
            require("ReceiverBindingFamily$A.access$002((dt29p1.ReceiverBindingFamily$A) arg1, arg2)" in text_body,
                    f"{class_name} BCI 17 lost physical accessor")
            require(any(segment["origin"]["primary"]["bci"] == 17 for segment in
                        method["source_map"]["segments"]), f"{class_name} BCI 17 lost source")
            source_bcis[class_name] = [2, 7, 12, 17]
            source_maps[class_name + ".set"] = mapped_sites(method, [2, 7, 12, 17])
        root = body(jar, "dt29p1/ReceiverBindingFamily", "run")
        root_bcis = [14, 31, 57]
        require(root["text"].count("bits((dt29p1.ReceiverBindingFamily$A)") == 3,
                "run lost one of three private bits(A) bindings")
        require(all(any(segment["origin"]["primary"]["bci"] == bci and
                        "bits((dt29p1.ReceiverBindingFamily$A)" in
                        root["text"][segment["start"]:segment["end"]]
                        for segment in root["source_map"]["segments"])
                    for bci in root_bcis), "run bits(A) source map lost a call")
        source_maps["run.bits(A)"] = mapped_sites(root, root_bcis)
        physical = run("javap", "-classpath", jar, "-c", "-p",
                       "dt29p1.ReceiverBindingFamily")
        require(physical.returncode == 0 and physical.stdout.count("Method bits:(Ldt29p1/ReceiverBindingFamily$A;)I") == 3,
                "run physical bits(A) call count changed")
        require("private static int bits(dt29p1.ReceiverBindingFamily$B);" in physical.stdout,
                "competing bits(B) overload absent")

        base = {name + ".class": (work / "original" / (name + ".class")).read_bytes()
                for name in names}
        a = "dt29p1/ReceiverBindingFamily$A.class"
        b = "dt29p1/ReceiverBindingFamily$B.class"
        c = "dt29p1/ReceiverBindingFamily$C.class"
        refused = {}
        cases = {
            "wrong_parent": {b: change_parent(base[b])},
            "wrong_owner": {c: change_fieldref(base[c], owner="dt29p1/ReceiverBindingFamily$C")},
            "wrong_name": {c: change_fieldref(base[c], name="set")},
            "wrong_descriptor": {c: change_fieldref(base[c], descriptor="I")},
            "wrong_ssa_receiver": {c: change_ssa_receiver(base[c])},
            "private": {a: change_field_flags(base[a], 0x0002)},
            "static": {a: change_field_flags(base[a], 0x0004 | 0x0008)},
            "final": {a: change_field_flags(base[a], 0x0004 | 0x0010)},
        }
        for label, overrides in cases.items():
            method = body(variant(work, label, base, overrides),
                          "dt29p1/ReceiverBindingFamily$C", "set")
            require("@bytecode 7" in method["text"] and
                    ").protectedField = arg2;" not in method["text"],
                    f"{label} unexpectedly published BCI 7")
            refused[label] = "refused at BCI 7"
        relocated = base.copy()
        del relocated[c]
        relocated["xt29p1/ReceiverBindingFamily$C.class"] = change_class_package(base[c])
        cross = variant(work, "cross_package", relocated, {})
        cross_body = body(cross, "xt29p1/ReceiverBindingFamily$C", "set")
        require("@bytecode 7" in cross_body["text"] and
                ").protectedField = arg2;" not in cross_body["text"],
                "cross-package protected owner cast escaped")
        refused["cross_package"] = "refused at BCI 7"
        accessor_ambiguous = variant(work, "ambiguous_accessor", base,
                                     {a: change_method_name(base[a], "all", "access$002")})
        accessor_body = body(accessor_ambiguous, "dt29p1/ReceiverBindingFamily$C", "set")
        require("@bytecode 17" in accessor_body["text"] and
                "access$002((dt29p1.ReceiverBindingFamily$A) arg1" not in accessor_body["text"],
                "ambiguous accessor was emitted")
        refused["ambiguous_accessor"] = "refused at BCI 17"
        bits_mismatch = variant(work, "bits_wrong_descriptor", base, {
            "dt29p1/ReceiverBindingFamily.class": change_method_descriptor(
                base["dt29p1/ReceiverBindingFamily.class"], "bits",
                "(Ldt29p1/ReceiverBindingFamily$B;)I")})
        bits_body = body(bits_mismatch, "dt29p1/ReceiverBindingFamily", "run")
        require(any("14" in line.split() for line in bits_body["text"].splitlines()
                    if "@bytecode" in line) and
                "bits((dt29p1.ReceiverBindingFamily$A)" not in bits_body["text"],
                "missing private bits(A) declaration was still selected")
        refused["bits_wrong_descriptor"] = "refused at BCI 14"
        bits_owner = variant(work, "bits_wrong_owner", base, {
            "dt29p1/ReceiverBindingFamily.class": change_bits_owner(
                base["dt29p1/ReceiverBindingFamily.class"])})
        bits_owner_body = body(bits_owner, "dt29p1/ReceiverBindingFamily", "run")
        require(any("14" in line.split() for line in bits_owner_body["text"].splitlines()
                    if "@bytecode" in line) and
                "bits((dt29p1.ReceiverBindingFamily$A)" not in bits_owner_body["text"],
                "wrong private method CP owner was still selected")
        refused["bits_wrong_owner"] = "refused at BCI 14"
        bits_name = variant(work, "bits_wrong_name", base, {
            "dt29p1/ReceiverBindingFamily.class": change_bits_name(
                base["dt29p1/ReceiverBindingFamily.class"])})
        bits_name_body = body(bits_name, "dt29p1/ReceiverBindingFamily", "run")
        require(any("14" in line.split() for line in bits_name_body["text"].splitlines()
                    if "@bytecode" in line) and
                "bits((dt29p1.ReceiverBindingFamily$A)" not in bits_name_body["text"],
                "wrong private method CP name was still selected")
        refused["bits_wrong_name"] = "refused at BCI 14"
        budget = run(JARDE, "class-source", "--input", jar, "--class",
                     "dt29p1/ReceiverBindingFamily$C", "--policy", "plain-jar",
                     "--release", "8", "--format", "json", "--budget", "method_bodies=1")
        require(budget.returncode == 4 and "stopped" in budget.stdout,
                "body budget stop was not explicit")
        result = {"fixture_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                  "jadx_head": JADX_HEAD, "physical_classes": names,
                  "original": original, "jadx": jadx_result, "jarde": jarde_result,
                  "source_bcis": source_bcis,
                  "source_maps": source_maps,
                  "run_bits_a_call_bcis": root_bcis,
                  "fixed_combined": fixed_combined(work),
                  "negative": refused,
                  "budget": "method_bodies=1 stopped explicitly"}
        (HERE / "results.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
