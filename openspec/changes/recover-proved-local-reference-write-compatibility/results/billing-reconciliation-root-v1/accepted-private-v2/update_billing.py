#!/usr/bin/env python3
"""Regenerate only p5 bulk AnalysisSteps pins from captured recorder output."""
from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

FIELDS = (
    "archive_entries", "entry_bytes", "class_bytes", "class_headers",
    "method_bodies", "ir_items", "analysis_steps", "result_items", "output_bytes",
)
CASES = (
    ("flat-mixed", "FLAT_MIXED"),
    ("nested-mixed", "NESTED_MIXED"),
    ("two-origins-one-identity", "TWO_ORIGINS"),
    ("many-method-class", "MANY_METHOD_CLASS"),
    ("damaged-tail", "DAMAGED_TAIL"),
    ("deep-expression", "DEEP_EXPRESSION"),
)
ARMS = (("direct", "DIRECT_ARM"), ("shared", "SHARED_ARM"))
CONST_RE = re.compile(r"const\s+(\w+):\s*Self\s*=\s*Self\s*\{(.*?)\n\s*\};", re.S)
FIELD_RE = re.compile(r"\b(" + "|".join(FIELDS) + r")\s*:\s*(\d+)\s*,")
LITERAL_FIELD_RE = re.compile(r"\b(" + "|".join(FIELDS) + r")\s*:\s*(\d+)\s*,?")
LITERAL_RE = re.compile(r"Billing\s*\{([^}]*)\}")
PAIR_RE = re.compile(r"//\s+([\w-]+)\s+[^\n]*\n\s*Billing\s*\{([^}]*)\}")
NUMBER_RE = re.compile(r"\b([a-z_]+)=(\d+)\b")


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def rust_constants(source: str) -> dict[str, dict[str, int]]:
    found: dict[str, dict[str, int]] = {}
    for name, body in CONST_RE.findall(source):
        if name in found:
            raise ValueError(f"duplicate Rust Billing constant: {name}")
        fields = dict((key, int(value)) for key, value in FIELD_RE.findall(body))
        if set(fields) != set(FIELDS) or len(FIELD_RE.findall(body)) != len(FIELDS):
            raise ValueError(f"{name} does not contain each Billing field exactly once")
        found[name] = fields
    return found


def parse_literal(body: str) -> dict[str, int]:
    fields = dict((key, int(value)) for key, value in LITERAL_FIELD_RE.findall(body))
    if set(fields) != set(FIELDS) or len(LITERAL_FIELD_RE.findall(body)) != len(FIELDS):
        raise ValueError("recorder Billing literal is incomplete or repeats a field")
    return fields


def case_records(text: str) -> dict[str, dict[str, int]]:
    records: dict[str, dict[str, int]] = {}
    for case, body in PAIR_RE.findall(text):
        if case not in {name for name, _ in CASES}:
            continue
        if case in records:
            raise ValueError(f"duplicate recorder row for {case}")
        records[case] = parse_literal(body)
    expected = {name for name, _ in CASES}
    if set(records) != expected:
        raise ValueError(f"recorder rows differ: missing={sorted(expected-records.keys())}, extra={sorted(records.keys()-expected)}")
    return records


def arm_records(text: str) -> dict[str, dict[str, int]]:
    records: dict[str, dict[str, int]] = {}
    for arm, rest in re.findall(r"(?m)^\s*(direct|shared)\s+requests=\d+\s+([^\n]+)$", text):
        if arm in records:
            raise ValueError(f"duplicate arm ledger for {arm}")
        records[arm] = {key: int(value) for key, value in NUMBER_RE.findall(rest)}
    if set(records) != {"direct", "shared"}:
        raise ValueError("raw output must contain exactly one direct and one shared arm ledger")
    return records


def block_for(source: str, name: str) -> tuple[int, int, dict[str, int]]:
    matches = [(m.start(), m.end(), m.group(2)) for m in CONST_RE.finditer(source) if m.group(1) == name]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one declaration for {name}; got {len(matches)}")
    start, end, body = matches[0]
    fields = dict((key, int(value)) for key, value in FIELD_RE.findall(body))
    if set(fields) != set(FIELDS) or len(FIELD_RE.findall(body)) != len(FIELDS):
        raise ValueError(f"{name} does not contain each Billing field exactly once")
    return start, end, fields


def verify_same_except_analysis(old: dict[str, int], new: dict[str, int], label: str) -> None:
    for field in FIELDS:
        if field == "analysis_steps":
            continue
        if old[field] != new[field]:
            raise ValueError(f"{label}: {field} changed {old[field]} -> {new[field]}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("recorder_stdout", type=Path, help="captured ignored record_the_billing_table stdout")
    parser.add_argument("arm_stdout", type=Path, help="captured actual direct/shared ledger stdout")
    parser.add_argument("--base", type=Path, default=Path(__file__).with_name("p5_bulk_corpus.base.rs"))
    parser.add_argument("--output", type=Path, default=Path(__file__).with_name("p5_bulk_corpus.candidate.rs"))
    args = parser.parse_args()

    original_bytes = args.base.read_bytes()
    source = original_bytes.decode("utf-8")
    original_sha = digest(original_bytes)
    constants = rust_constants(source)
    cases = case_records(args.recorder_stdout.read_text(encoding="utf-8", errors="replace"))
    arms = arm_records(args.arm_stdout.read_text(encoding="utf-8", errors="replace"))

    updates: list[tuple[str, str, int, int]] = []
    old_by_case: dict[str, int] = {}
    for row, constant in CASES:
        if constant not in constants:
            raise ValueError(f"missing old constant {constant}")
        _, _, old = block_for(source, constant)
        measured = cases[row]
        verify_same_except_analysis(old, measured, row)
        old_by_case[row] = old["analysis_steps"]
        updates.append((constant, row, old["analysis_steps"], measured["analysis_steps"]))

    for arm, constant in ARMS:
        if constant not in constants:
            raise ValueError(f"missing old constant {constant}")
        _, _, old = block_for(source, constant)
        observed = arms[arm]
        required = {"archive_entries", "entry_bytes", "class_bytes", "class_headers", "method_bodies", "ir_items", "analysis_steps", "result_items", "output_bytes"}
        if not required.issubset(observed):
            raise ValueError(f"{arm} ledger is missing required Billing dimensions")
        measured = {field: observed[field] for field in FIELDS}
        verify_same_except_analysis(old, measured, arm)
        updates.append((constant, arm, old["analysis_steps"], measured["analysis_steps"]))

    # These two deltas are independently established by the fullroot-v3 failed fixed-pin run.
    if cases["flat-mixed"]["analysis_steps"] - old_by_case["flat-mixed"] != 8:
        raise ValueError("flat-mixed recorder does not match the observed +8 AnalysisSteps delta")
    if any(arms[arm]["analysis_steps"] - constants[name]["analysis_steps"] != 226 for arm, name in ARMS):
        raise ValueError("direct/shared ledgers do not match the observed +226 AnalysisSteps delta")

    candidate = source
    for constant, label, old_value, new_value in updates:
        start, end, _ = block_for(candidate, constant)
        block = candidate[start:end]
        needle = re.compile(r"(\banalysis_steps\s*:\s*)" + str(old_value) + r"(\s*,)")
        block2, count = needle.subn(r"\g<1>" + str(new_value) + r"\g<2>", block)
        if count != 1:
            raise ValueError(f"{constant} old AnalysisSteps value {old_value} is not unique")
        candidate = candidate[:start] + block2 + candidate[end:]

    changes = [f"{row} {old}->{new}" for _, row, old, new in updates]
    explanation = (
        "// 2026-10-12 local-reference-write-compatibility bills the physical Object-write scan.\n"
        "// AnalysisSteps: +8/+8/+210 on flat/nested/many; +226 on each arm.\n"
        "// All other rows and Billing dimensions are unchanged.\n"
    )
    anchor = "impl Billing {\n    /// `flat-mixed`"
    if candidate.count(anchor) != 1:
        raise ValueError("cannot find the unique case-table comment insertion point")
    candidate = candidate.replace(anchor, explanation + anchor, 1)
    args.output.write_text(candidate, encoding="utf-8")
    print(f"base_sha256={original_sha}")
    print(f"candidate_sha256={digest(candidate.encode('utf-8'))}")
    for constant, label, old_value, new_value in updates:
        print(f"{label}: AnalysisSteps {old_value} -> {new_value}")
    print(f"candidate={args.output}")


if __name__ == "__main__":
    main()
