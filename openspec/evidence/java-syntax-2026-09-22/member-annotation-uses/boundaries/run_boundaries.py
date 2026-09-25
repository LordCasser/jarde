#!/usr/bin/env python3
"""Build legal Java 8 boundary fixtures and controlled classfile patches."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import struct
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
OUT = HERE / "generated"
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-ann-fd-root-final"))
CLI_SHA256 = "f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], log: Path) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + ".exit").write_text(f"{result.returncode}\n")
    return result


class Reader:
    def __init__(self, data: bytes, offset: int = 0):
        self.data = data
        self.pos = offset

    def u1(self) -> int:
        value = self.data[self.pos]
        self.pos += 1
        return value

    def u2(self) -> int:
        value = struct.unpack_from(">H", self.data, self.pos)[0]
        self.pos += 2
        return value

    def u4(self) -> int:
        value = struct.unpack_from(">I", self.data, self.pos)[0]
        self.pos += 4
        return value


def parse_class(data: bytes) -> tuple[list[str | None], list[dict]]:
    r = Reader(data)
    assert r.u4() == 0xCAFEBABE
    r.u2(); r.u2()
    cp_count = r.u2()
    cp: list[str | None] = [None] * cp_count
    i = 1
    while i < cp_count:
        tag = r.u1()
        if tag == 1:
            size = r.u2()
            cp[i] = data[r.pos:r.pos + size].decode("utf-8", "replace")
            r.pos += size
        elif tag in (3, 4):
            r.pos += 4
        elif tag in (5, 6):
            r.pos += 8
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            r.pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            r.pos += 4
        elif tag == 15:
            r.pos += 3
        else:
            raise ValueError(f"unsupported constant pool tag {tag}")
        i += 1
    r.pos += 6
    interface_count = r.u2()
    r.pos += 2 * interface_count
    members = []
    for section in ("field", "method"):
        for _ in range(r.u2()):
            access = r.u2()
            name_i = r.u2()
            desc_i = r.u2()
            attrs = []
            for _ in range(r.u2()):
                name_index = r.u2()
                length = r.u4()
                start = r.pos
                attrs.append({"name": cp[name_index], "length": length,
                              "start": start, "end": start + length})
                r.pos += length
            members.append({"section": section, "access": access,
                            "name": cp[name_i], "descriptor": cp[desc_i],
                            "attributes": attrs})
    return cp, members


def annotation_end(data: bytes, offset: int) -> int:
    r = Reader(data, offset)
    r.u2()  # type_index
    for _ in range(r.u2()):
        r.u2()  # element_name_index
        r.pos = element_value_end(data, r.pos)
    return r.pos


def element_value_end(data: bytes, offset: int) -> int:
    r = Reader(data, offset)
    tag = chr(r.u1())
    if tag in "BCDFIJSZsc":
        r.u2()
    elif tag == "e":
        r.u2(); r.u2()
    elif tag == "@":
        return annotation_end(data, r.pos)
    elif tag == "[":
        for _ in range(r.u2()):
            r.pos = element_value_end(data, r.pos)
    else:
        raise ValueError(f"unsupported element_value tag {tag}")
    return r.pos


def attr(member: dict, name: str) -> dict:
    matches = [a for a in member["attributes"] if a["name"] == name]
    assert len(matches) == 1, (member["name"], name, matches)
    return matches[0]


def record_spans(class_path: Path) -> list[dict]:
    data = class_path.read_bytes()
    cp, members = parse_class(data)
    del cp
    spans = []
    for member in members:
        for a in member["attributes"]:
            if a["name"] in ("RuntimeInvisibleAnnotations",
                             "RuntimeInvisibleParameterAnnotations"):
                spans.append({"member": f"{member['section']} {member['name']}{member['descriptor']}",
                              "attribute": a["name"], "classfile_offset": a["start"],
                              "content_length": a["length"],
                              "content_end_exclusive": a["end"]})
    return spans


def duplicate_field_annotation(source: bytes, target: Path) -> dict:
    _, members = parse_class(source)
    field = next(m for m in members if m["section"] == "field" and m["name"] == "field")
    a = attr(field, "RuntimeInvisibleAnnotations")
    payload = source[a["start"]:a["end"]]
    count = struct.unpack_from(">H", payload, 0)[0]
    assert count == 1
    first_end = annotation_end(payload, 2)
    assert first_end == len(payload)
    changed = struct.pack(">H", 2) + payload[2:] + payload[2:first_end]
    patched = bytearray(source[:a["start"] - 4] + struct.pack(">I", len(changed)) + changed + source[a["end"]:])
    # Attribute layout is u2 name_index, u4 length, then info; retain name_index.
    target.write_bytes(patched)
    return {"attribute": a["name"], "old_annotation_count": count,
            "new_annotation_count": 2, "old_content_length": len(payload),
            "new_content_length": len(changed), "patch": "duplicated one encoded annotation at same field position"}


def shorten_parameter_attribute(source: bytes, target: Path) -> dict:
    _, members = parse_class(source)
    method = next(m for m in members if m["section"] == "method" and m["name"] == "wideAndVarargs")
    assert method["descriptor"] == "(J D[Ljava/lang/String;)I".replace(" ", "")
    a = attr(method, "RuntimeInvisibleParameterAnnotations")
    payload = source[a["start"]:a["end"]]
    parameter_count = payload[0]
    assert parameter_count == 3
    r = Reader(payload, 1)
    ends = []
    for _ in range(parameter_count):
        for _ in range(r.u2()):
            r.pos = annotation_end(payload, r.pos)
        ends.append(r.pos)
    assert r.pos == len(payload)
    shortened = bytes([2]) + payload[1:ends[1]]
    header = a["start"] - 4
    patched = source[:header] + struct.pack(">I", len(shortened)) + shortened + source[a["end"]:]
    target.write_bytes(patched)
    return {"attribute": a["name"], "descriptor_parameter_count": 3,
            "old_attribute_parameter_count": parameter_count,
            "new_attribute_parameter_count": 2,
            "old_content_length": len(payload), "new_content_length": len(shortened),
            "patch": "removed final parameter group and updated u1 count and attribute_length"}


def main() -> None:
    assert CLI.is_file() and sha(CLI) == CLI_SHA256
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir()
    legal = OUT / "legal"
    legal.mkdir()
    sources = [HERE / "BoundaryTagged.java", HERE / "BoundaryRunner.java"]
    compile_result = run(["javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", str(legal),
                          *(str(path) for path in sources)], OUT / "legal-javac.log")
    assert compile_result.returncode == 0
    tagged = legal / "BoundaryTagged.class"
    runner = legal / "BoundaryRunner.class"
    assert run(["java", "-Xverify:all", "-cp", str(legal), "BoundaryRunner"],
               OUT / "legal-run.txt").returncode == 0
    assert run(["javap", "-v", "-c", "-p", str(tagged)], OUT / "javap-legal.txt").returncode == 0

    patches = OUT / "patched"
    duplicate_dir = patches / "duplicate-same-position"
    mismatch_dir = patches / "parameter-count-mismatch"
    duplicate_dir.mkdir(parents=True)
    mismatch_dir.mkdir(parents=True)
    shutil.copy2(runner, duplicate_dir / runner.name)
    shutil.copy2(runner, mismatch_dir / runner.name)
    duplicate_record = duplicate_field_annotation(tagged.read_bytes(), duplicate_dir / tagged.name)
    mismatch_record = shorten_parameter_attribute(tagged.read_bytes(), mismatch_dir / tagged.name)
    for name, directory in (("duplicate-same-position", duplicate_dir),
                            ("parameter-count-mismatch", mismatch_dir)):
        run(["javap", "-v", "-c", "-p", str(directory / tagged.name)], OUT / f"javap-{name}.txt")
        run(["java", "-Xverify:all", "-cp", str(directory), "BoundaryRunner"],
            OUT / f"{name}-run.txt")
        base = [str(CLI), "class-source", "--input", str(directory / tagged.name),
                "--class", "BoundaryTagged", "--policy", "single-class", "--release", "8"]
        run([*base, "--format", "text"], OUT / f"jarde-{name}-text.log")
        run([*base, "--format", "json", "--evidence", "all"],
            OUT / f"jarde-{name}-json.log")

    # JADX is run on the legal source class; patched input results are preserved
    # through javap, verification, and the frozen Jarde reader.
    run(["jadx", "-d", str(OUT / "jadx"), str(tagged), str(runner)], OUT / "jadx-legal.log")
    base = [str(CLI), "class-source", "--input", str(tagged), "--class", "BoundaryTagged",
            "--policy", "single-class", "--release", "8"]
    for fmt, args in (("text", ["--format", "text"]),
                      ("json", ["--format", "json", "--evidence", "all"])):
        run([*base, *args], OUT / f"jarde-legal-{fmt}.log")
    (OUT / "summary.json").write_text(json.dumps({
        "javac": compile_result.args,
        "cli_sha256": CLI_SHA256,
        "source_sha256": {p.name: sha(p) for p in sources},
        "class_sha256": {p.name: sha(p) for p in legal.glob("*.class")},
        "classfile_size": {p.name: p.stat().st_size for p in legal.glob("*.class")},
        "legal_runtime_stdout": (OUT / "legal-run.txt").read_text().splitlines(),
        "legal_attribute_spans": record_spans(tagged),
        "controlled_patches": {
            "duplicate-same-position": {**duplicate_record,
                "patched_sha256": sha(duplicate_dir / tagged.name),
                "attribute_spans": record_spans(duplicate_dir / tagged.name),
                "java_verify_exit": (OUT / "duplicate-same-position-run.txt.exit").read_text().strip(),
                "javap_exit": (OUT / "javap-duplicate-same-position.txt.exit").read_text().strip()},
            "parameter-count-mismatch": {**mismatch_record,
                "patched_sha256": sha(mismatch_dir / tagged.name),
                "attribute_spans": record_spans(mismatch_dir / tagged.name),
                "java_verify_exit": (OUT / "parameter-count-mismatch-run.txt.exit").read_text().strip(),
                "javap_exit": (OUT / "javap-parameter-count-mismatch.txt.exit").read_text().strip()},
        },
    }, indent=2) + "\n")
    assert sha(CLI) == CLI_SHA256


if __name__ == "__main__":
    main()
