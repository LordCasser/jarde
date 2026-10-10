#!/usr/bin/env python3
"""Register exactly the reviewed A1 expected source in the P5 corpus manifest."""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import os
import re
import tempfile
from pathlib import Path
from typing import Any

import blake3


TARGET = "tests/fixtures/p3-array-slot-retype-locals/expected/A1.static-init.jarde.java"
TARGET_BYTES = 2210
TARGET_SHA256 = "a8b84e8cfd3fa4610a05ecd9ef3f1204eb267a063aa3fcd0d28dcc6d73cc96ca"
TARGET_BLAKE3 = "eaac73c5450923dadab849829f1dc2ca1e9503b0c5c3aa14fbd7ab39df6cc7b4"
EXPECTED_SCANNED_FILES = 2062
EXPECTED_PRIOR_MANIFEST_FILES = 2061
RESULT_RELATIVE = (
    "openspec/changes/preserve-nonfinal-static-initializer-phase/results/"
    "register-a1-fingerprint-root-v1.json"
)
RUST_TEST = "tests/p5_corpus_fingerprint.rs"
RUST_GENERATOR = (
    "cargo test --test p5_corpus_fingerprint --locked -- "
    "--ignored regenerate_corpus_fingerprint"
)


def fail(message: str) -> None:
    raise SystemExit(f"refusing corpus registration: {message}")


def rust_string(source: str, name: str) -> str:
    match = re.search(rf'const {re.escape(name)}: &str = ("(?:\\.|[^"\\])*");', source)
    if match is None:
        fail(f"cannot read Rust constant {name}")
    return json.loads(match.group(1))


def rust_string_slice(source: str, name: str) -> list[str]:
    match = re.search(
        rf"const {re.escape(name)}: &\[&str\] = &\[(.*?)\];", source, re.DOTALL
    )
    if match is None:
        fail(f"cannot read Rust scope constant {name}")
    values = re.findall(r'"(?:\\.|[^"\\])*"', match.group(1))
    return [json.loads(value) for value in values]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def collect(root: Path, roots: list[str], excluded_dirs: list[str], excluded_names: list[str],
            excluded_extensions: list[str]) -> list[str]:
    paths: list[str] = []

    def visit(directory: Path) -> None:
        try:
            entries = sorted(os.scandir(directory), key=lambda entry: entry.name)
        except OSError as error:
            fail(f"cannot scan {directory}: {error}")
        for entry in entries:
            path = Path(entry.path)
            try:
                if entry.is_dir(follow_symlinks=False):
                    if entry.name not in excluded_dirs:
                        visit(path)
                    continue
                if not entry.is_file(follow_symlinks=False) or entry.name in excluded_names:
                    continue
            except OSError as error:
                fail(f"cannot inspect {path}: {error}")
            extension = path.suffix[1:] if path.suffix else ""
            if extension in excluded_extensions:
                continue
            paths.append(path.relative_to(root).as_posix())

    for relative in roots:
        directory = root / relative
        if directory.is_symlink() or not directory.is_dir():
            fail(f"corpus root is missing or is a symlink: {relative}")
        visit(directory)
    return sorted(paths)


def fingerprint(root: Path, paths: list[str]) -> list[dict[str, Any]]:
    entries = []
    for relative in paths:
        data = (root / relative).read_bytes()
        entries.append(
            {"blake3": blake3.blake3(data).hexdigest(), "bytes": len(data), "path": relative}
        )
    return entries


def atomic_write(path: Path, data: bytes) -> None:
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def pretty_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def main() -> None:
    root = Path(__file__).resolve().parents[4]
    rust_path = root / RUST_TEST
    manifest_path = root / "tests/fixtures/corpus-fingerprint.json"
    result_path = root / RESULT_RELATIVE
    if result_path.exists():
        fail(f"refusing to overwrite {result_path.relative_to(root)}")

    rust_raw = rust_path.read_bytes()
    rust_source = rust_raw.decode("utf-8")
    manifest_relative = rust_string(rust_source, "MANIFEST")
    schema = rust_string(rust_source, "SCHEMA")
    roots = rust_string_slice(rust_source, "ROOTS")
    excluded_dirs = rust_string_slice(rust_source, "EXCLUDED_DIRECTORIES")
    excluded_names = rust_string_slice(rust_source, "EXCLUDED_FILE_NAMES")
    excluded_extensions = rust_string_slice(rust_source, "EXCLUDED_EXTENSIONS")
    if manifest_relative != "tests/fixtures/corpus-fingerprint.json":
        fail(f"unexpected Rust manifest pin: {manifest_relative}")
    if roots != ["tests/fixtures", "fuzz/corpus"]:
        fail(f"unexpected Rust corpus roots: {roots}")
    if excluded_dirs != ["target", "out", "artifacts", "__pycache__"]:
        fail(f"unexpected Rust excluded directories: {excluded_dirs}")
    if excluded_names != ["corpus-fingerprint.json"]:
        fail(f"unexpected Rust excluded filenames: {excluded_names}")
    if excluded_extensions != ["md", "py"]:
        fail(f"unexpected Rust excluded extensions: {excluded_extensions}")

    original_raw = manifest_path.read_bytes()
    try:
        original = json.loads(original_raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"invalid original manifest JSON: {error}")
    # Refuse serializer drift before constructing or writing the revised document.
    if pretty_json(original) != original_raw:
        fail("original manifest does not round-trip byte-exactly through pretty JSON")
    if original.get("schema") != schema:
        fail(f"manifest schema differs from Rust pin: {original.get('schema')!r}")
    scope = original.get("scope")
    if not isinstance(scope, dict) or (
        scope.get("roots") != roots
        or scope.get("excluded_directories") != excluded_dirs
        or scope.get("excluded_file_names") != excluded_names
        or scope.get("excluded_extensions") != excluded_extensions
    ):
        fail("manifest scope differs from the Rust test's actual scope constants")

    actual_paths = collect(root, roots, excluded_dirs, excluded_names, excluded_extensions)
    if len(actual_paths) != EXPECTED_SCANNED_FILES:
        fail(f"expected {EXPECTED_SCANNED_FILES} scanned corpus files, found {len(actual_paths)}")
    actual = fingerprint(root, actual_paths)
    target_entries = [entry for entry in actual if entry["path"] == TARGET]
    if target_entries != [{"blake3": TARGET_BLAKE3, "bytes": TARGET_BYTES, "path": TARGET}]:
        fail(f"the new fixture does not match its reviewed byte pins: {target_entries}")
    target_raw = (root / TARGET).read_bytes()
    if sha256(target_raw) != TARGET_SHA256:
        fail("the new fixture SHA-256 differs from the reviewed pin")

    prior_files = original.get("files")
    if not isinstance(prior_files, list) or len(prior_files) != EXPECTED_PRIOR_MANIFEST_FILES:
        fail(f"expected {EXPECTED_PRIOR_MANIFEST_FILES} prior manifest entries")
    prior_by_path: dict[str, dict[str, Any]] = {}
    for entry in prior_files:
        if not isinstance(entry, dict) or not isinstance(entry.get("path"), str):
            fail("prior manifest contains a malformed file entry")
        path = entry["path"]
        if path in prior_by_path:
            fail(f"prior manifest repeats {path}")
        prior_by_path[path] = entry
    actual_by_path = {entry["path"]: entry for entry in actual}
    unlisted = sorted(set(actual_by_path) - set(prior_by_path))
    missing = sorted(set(prior_by_path) - set(actual_by_path))
    if unlisted != [TARGET] or missing:
        fail(f"corpus path drift; unlisted={unlisted}, missing={missing}")
    mismatches = [
        path for path, entry in prior_by_path.items() if entry != actual_by_path[path]
    ]
    if mismatches:
        fail(f"existing corpus pins differ from current bytes: {mismatches[:8]}")

    updated = dict(original)
    updated["files"] = sorted([*prior_files, target_entries[0]], key=lambda entry: entry["path"])
    for key, value in original.items():
        if key != "files" and updated.get(key) != value:
            fail(f"classification changed while preparing update: {key}")
    updated_raw = pretty_json(updated)
    after_roundtrip = json.loads(updated_raw)
    if after_roundtrip != updated:
        fail("updated manifest failed JSON round-trip validation")
    if [entry for entry in updated["files"] if entry["path"] != TARGET] != prior_files:
        fail("updated file list changed an existing manifest row")
    if updated["files"] != actual:
        fail("updated file list does not exactly equal the complete current corpus scan")

    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    atomic_write(manifest_path, updated_raw)
    result = {
        "schema": "jarde-a1-corpus-registration/1",
        "status": "registered",
        "manifest": manifest_relative,
        "manifest_sha256_before": sha256(original_raw),
        "manifest_sha256_after": sha256(updated_raw),
        "rust_scope_source_sha256": sha256(rust_raw),
        "blake3_package_version": importlib.metadata.version("blake3"),
        "corpus_comparison": {
            "scanned_files": len(actual),
            "manifest_files_before": len(prior_files),
            "manifest_files_after": len(updated["files"]),
            "unchanged_existing_entries": len(prior_files),
            "scanned_entries_sha256": sha256(pretty_json(actual)),
            "updated_files_array_sha256": sha256(pretty_json(updated["files"])),
            "added_only": target_entries[0],
            "unlisted_before": unlisted,
            "missing_before": missing,
            "existing_pin_mismatches": mismatches,
        },
        "rust_generator": {
            "command": RUST_GENERATOR,
            "status": "not_executed_by_this_script",
            "stdout": None,
            "stderr": None,
            "note": "The Python registrar does not invoke the Rust generator; the root wrapper records its argv and streams separately.",
        },
    }
    atomic_write(result_path, pretty_json(result))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
