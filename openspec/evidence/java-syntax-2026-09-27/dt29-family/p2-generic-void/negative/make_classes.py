#!/usr/bin/env python3
"""Build verifier-valid negative class files for the DT-29 P2 projection gate."""

from __future__ import annotations

import importlib.util
import pathlib
import shutil
import sys


root = pathlib.Path(sys.argv[1])
classes = pathlib.Path(sys.argv[2])
output = pathlib.Path(sys.argv[3])
patcher_path = root / "openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/build_evidence.py"
spec = importlib.util.spec_from_file_location("signature_patcher", patcher_path)
patcher = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(patcher)

output.mkdir(parents=True, exist_ok=True)
source = classes / "dt29p2/Setter.class"
old = b"<T:Ldt29p2/Bound;>(TT;Z)V"
for name, replacement in {
    "wrong-bound": b"<T:Ljava/lang/Object;>(TT;Z)V",
    "unbound-variable": b"<T:Ldt29p2/Bound;>(TU;Z)V",
}.items():
    (output / f"{name}.class").write_bytes(
        patcher.replace_utf8(source.read_bytes(), old, replacement)
    )

for source_name, class_name, signature in (
    (
        "IncompatibleSetter.class",
        "IncompatibleSetter",
        b"<T:Ldt29p2/Bound;>(TT;Z)V",
    ),
    (
        "OverloadedSetter.class",
        "OverloadedSetter",
        b"<T:Ldt29p2/Bound;:Ldt29p2/ExtraBound;>(TT;Z)V",
    ),
):
    data = (classes / "dt29p2" / source_name).read_bytes()
    (output / f"{class_name}.class").write_bytes(
        patcher.add_method_signature(data, b"(Ldt29p2/Bound;Z)V", signature)
    )
