#!/usr/bin/env python3
import hashlib, json
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
OUT = HERE / "ordinary-static-regression-root-v2"
RESULT = HERE / "ordinary-static-regression-root-acceptance-v1.json"
def sha(data): return hashlib.sha256(data).hexdigest()
def data(item):
    path = Path(item["path"])
    payload = (path if path.is_absolute() else OUT / path).read_bytes()
    assert len(payload) == item["bytes"] and sha(payload) == item["sha256"]
    return payload
assert not RESULT.exists()
manifest_bytes = (OUT / "manifest.json").read_bytes()
inventory_bytes = (OUT / "file-inventory.json").read_bytes()
m = json.loads(manifest_bytes); inv = json.loads(inventory_bytes)
assert m["status"] == "completed" and m["failures"] == []
assert len(inv) == len({r["path"] for r in inv})
assert {r["path"] for r in inv} == {str(p.relative_to(OUT)) for p in OUT.rglob("*") if p.is_file() and p.name != "file-inventory.json"}
for row in inv: data(row)
assert all(r["ok"] for r in m["preflight"])
assert len(m["commands"]) == 24
for row in m["commands"]:
    assert row["exit"] == 0
    data(row["stdout"]); data(row["stderr"])
assert len(m["cases"]) == 6 and all(c["success"] for c in m["cases"])
original = {c["jdk_leg"]: c for c in m["cases"] if c["kind"] == "original"}
assert set(original) == {"javac8", "javac23"}
def raw(c): return (c["runtime"]["exit"], data(c["runtime"]["stdout"]), data(c["runtime"]["stderr"]))
assert raw(original["javac8"]) == raw(original["javac23"])
texts = []
for case in m["cases"]:
    argv = case["compile"]["argv"]
    assert argv[argv.index("-classpath")+1] == argv[argv.index("-sourcepath")+1]
    assert list(Path(argv[argv.index("-classpath")+1]).iterdir()) == []
    assert "-g" in argv and "-g:none" not in argv
    assert "-Xverify:all" in case["runtime"]["argv"]
    cp = Path(case["runtime"]["argv"][case["runtime"]["argv"].index("-cp")+1])
    assert {p.name for p in cp.rglob("*.class")} == {"OrdinaryInit.class", "Runner.class"}
    assert {Path(r["path"]).name for r in case["source_files"]} == {"OrdinaryInit.java", "Runner.java"}
    for row in case["source_files"] + case["classes"]: data(row)
    assert raw(case) == raw(original[case["jdk_leg"]])
    if case["kind"] == "rendered":
        assert all(case["checks"].values()) and case["generated_text_unchanged"]
        document = json.loads(data(case["document"]))
        assert data(case["render"]["stdout"]) == data(case["document"])
        source = next(row for row in case["source_files"] if Path(row["path"]).name == "OrdinaryInit.java")
        assert data(source) == document["text"].encode()
        texts.append(document["text"])
        fields = document["fields"]; proofs = document["initializer_proof"]["fields"]
        assert [p["write_order"] for p in proofs] == [0,1]
        assert [p["field_index"] for p in proofs] == [f["item"]["index"] for f in fields]
        assert [f["item"]["access_flags"] for f in fields] == [8,8]
        method = next(x for x in document["methods"] if bytes(x["item"]["name"]["raw"]) == b"<clinit>")
        report = method["outcome"]["report"]
        origins = [o for s in report["source_map"]["segments"] for o in [s["origin"]["primary"],*s["origin"]["derived"]] if o is not None]
        assert all(o["method"] == method["item"]["identity"] for o in origins)
        assert {p["write_bci"] for p in proofs} <= {o["bci"] for o in origins}
assert len(texts) == 4 and len(set(texts)) == 1
assert data(m["regression_test_source"]) == (ROOT / "src/enum_constants.rs").read_bytes()
result = {"status":"accepted", "manifest_sha256":sha(manifest_bytes), "inventory_sha256":sha(inventory_bytes), "file_count":len(inv), "command_count":24, "original_legs":2, "candidate_legs":4, "all_raw_equal":True, "all_text_equal":True, "test_source_sha256":m["regression_test_source"]["sha256"], "claim_boundary":"Frozen production CLI full-source replay plus independently executed exact Rust regression test; CLI does not serialize Engine enum_constant_proof."}
RESULT.write_text(json.dumps(result,indent=2)+"\n")
print(json.dumps(result))
