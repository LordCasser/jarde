#!/usr/bin/env python3
"""Read-only independent audit of GC09 field and constructor CLI8 replays."""
import hashlib
import json
import shlex
import zipfile
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
CHANGE = HERE.parents[1]
RESULTS = CHANGE / "results/candidate"
FIELD = RESULTS / "gc09-field-23-v9"
CTOR = RESULTS / "gc09-constructor-80-v8"
BASELINE = Path("/private/tmp/jarde-raw-receiver-final-v3-cli")
CANDIDATE = Path("/private/tmp/jarde-generic-calls-candidate-v8-cli")
BASE_SHA = "3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70"
CAND_SHA = "8770d823c70e7f61749cac836e468c0a991093822d1926822d4769adc1cf7339"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def lines(path):
    return Path(path).read_text(errors="replace").splitlines()


def main():
    errors, metrics = [], {}

    def require(ok, why):
        if not ok:
            errors.append(why)

    def same_hash(path, expected, label):
        p = Path(path)
        require(p.is_file(), label + " missing: " + str(p))
        if p.is_file():
            require(sha(p) == expected, label + " sha256: " + str(p))

    # Fixed CLI identities and baseline identity.
    same_hash(BASELINE, BASE_SHA, "baseline CLI")
    same_hash(CANDIDATE, CAND_SHA, "candidate CLI")
    build_preflight = read(HERE / "gc09-cli8-replay-preflight.json")
    build_snapshot = read(CHANGE / "results/local-gates/source-snapshot-v41.json")
    build_cli = read(CHANGE / "results/local-gates/candidate-cli-v8.json")
    same_hash(build_preflight["source_archive"], build_preflight["source_archive_sha256"], "CLI8 source archive")
    same_hash(build_preflight["source_snapshot_manifest"], build_preflight["source_snapshot_manifest_sha256"], "CLI8 source snapshot manifest")
    same_hash(build_preflight["candidate_cli_metadata"], build_preflight["candidate_cli_metadata_sha256"], "CLI8 build metadata")
    require(build_preflight["candidate_cli_sha256"] == CAND_SHA == build_cli["sha256"], "CLI8 pre-run artifact identity")
    require(build_preflight["source_archive_sha256"] == build_snapshot["archive_sha256"] == build_cli["source_archive_sha256"],
            "CLI8 source archive lineage")
    require(build_preflight["source_hashes"] == build_snapshot["files"] == build_cli["source_hashes"],
            "CLI8 source file hashes lineage")

    # Field replay provenance: outer invocation, preflight, exact executable snapshots.
    fo = read(FIELD / "outer-run.json")
    fp = read(FIELD / "preflight.json")
    fm = read(FIELD / "adapter-run/run-metadata.json")
    require(fo["exit"] == 0, "field outer exit")
    require(sha(fo["stdout"]) == fo["stdout_sha256"], "field outer stdout hash")
    require(sha(fo["stderr"]) == fo["stderr_sha256"], "field outer stderr hash")
    require(sha(fo["preflight"]) == fo["preflight_sha256"], "field outer preflight hash")
    require(fp["candidate_cli"]["sha256"] == CAND_SHA and fp["baseline_cli"]["sha256"] == BASE_SHA, "field preflight CLI identity")
    require(fm["candidate_cli_sha256"] == CAND_SHA and fm["baseline_cli_sha256"] == BASE_SHA, "field runner CLI identity")
    require(fo["argv"][1] == str((CHANGE / "results/field-23-replay-adapter.py").resolve()), "field outer actual adapter argv")
    require(fo["argv"][-2:] == ["--out", str((FIELD / "adapter-run").resolve())], "field outer output argv")
    for pathkey, hashkey, label in (("wrapper_snapshot", "wrapper_sha256", "field wrapper snapshot"),
                                   ("adapter_snapshot", "adapter_snapshot_sha256", "field adapter snapshot"),
                                   ("runner_snapshot", "runner_snapshot_sha256", "field runner snapshot")):
        same_hash(fp[pathkey], fp[hashkey], label)
    same_hash(fm["adapter_snapshot"], fm["adapter_snapshot_sha256"], "field executed adapter")
    same_hash(fm["runner_snapshot"], fm["runner_snapshot_sha256"], "field executed runner")
    same_hash(fp["source_audit"]["path"], fp["source_audit"]["sha256"], "field source audit manifest")
    same_hash(fp["reflect_driver_source"]["path"], fp["reflect_driver_source"]["sha256"], "field reflection driver source")
    for tool in fp["tools"]:
        same_hash(tool["path"], tool["sha256"], "field tool " + tool["label"])
    for v in fp["tool_version_commands"]:
        # Legacy Corretto 8 jar has no help/version option; its captured -help usage exits 1.
        require(v["exit"] == 0 or (v["label"] == "javac8_jar" and v["argv"][-1] == "-help" and v["exit"] == 1),
                "field version command " + v["label"])
    field_families = {x["family"]: x for x in fm["source_families"]}
    require(len(field_families) == 23, "field 23 source family count")
    for row in field_families.values():
        same_hash(row["path"], row["sha256"], "field source " + row["family"])
    audit_doc = read(fp["source_audit"]["path"])
    audited_sources = 0
    for row in field_families.values():
        rel = Path(row["path"]).relative_to(Path.cwd()).as_posix()
        expected = audit_doc.get("source_sha256", {}).get(rel)
        if expected is not None:
            audited_sources += 1
            require(expected == row["sha256"], "field frozen source audit lineage " + row["family"])
    require(audited_sources == 14, "field accepted historical source lineage count")
    field_hist = fm["historical_input_jars"]
    require(len(field_hist) == 28, "field historical JAR mapping count")
    for jar in field_hist:
        same_hash(jar["path"], jar["sha256"], "field historical JAR " + jar["family"])
        with zipfile.ZipFile(jar["path"]) as zf:
            entries = sorted(n for n in zf.namelist() if n.endswith(".class"))
            require(entries == [jar["family"] + ".class"], "field historical class mapping " + jar["family"])
            if entries:
                require(hashlib.sha256(zf.read(entries[0])).hexdigest() == jar["class_sha256"][entries[0]], "field historical class hash " + jar["family"])
    field_cmds = fm["actual_commands"]
    for c in field_cmds:
        for stream in ("stdout", "stderr"):
            same_hash(c[stream], c[stream + "_sha256"], "field command " + stream)
        argv = c["argv"]
        if Path(argv[0]).name == "javac":
            require("-classpath" in argv and "-sourcepath" in argv, "field javac isolated argv")
            if "-classpath" in argv and "-sourcepath" in argv:
                adapter_root = FIELD / "adapter-run"
                require(argv[argv.index("-classpath") + 1] == str(adapter_root / "empty-classpath"), "field javac empty classpath")
                require(argv[argv.index("-sourcepath") + 1] == str(adapter_root / "empty-sourcepath"), "field javac empty sourcepath")
        if Path(argv[0]).name == "java":
            cp_opt = next((i for i, x in enumerate(argv[:-1]) if x in ("-cp", "-classpath")), None)
            require(cp_opt is not None and "-Xverify:all" in argv, "field JVM verification argv")
            if cp_opt is not None:
                require(".jar" not in argv[cp_opt + 1].lower(), "field runtime JAR isolation")
    require(len(field_cmds) > 0, "field actual command transcript missing")

    # Recompute field outcomes from actual runner stdout/stderr files, not summary flags.
    fsummary = read(FIELD / "adapter-run/root/summary.json")
    frows = {(r["leg"], r["class"], r["version"]): r for r in fsummary}
    require(len(fsummary) == 92 and len(frows) == 92, "field summary matrix cardinality")
    api_prefixes = ("fieldGenericType=", "classvars=", "method=")
    fcounts = Counter()
    f_regressions = []
    for (leg, family, version), row in frows.items():
        case = FIELD / "adapter-run/root" / leg / family
        stem = "candidate" if version == "candidate" else "baseline"
        compile_out = lines(case / (stem + "-compile.stdout"))
        compile_err = lines(case / (stem + "-compile.stderr"))
        expected_compile = row["compile"]
        require(expected_compile == 0 or (family == "SCGB" and expected_compile == 1),
                f"field unexpected compile status {leg}/{family}/{version}: {expected_compile}")
        # Tie saved per-case compiler streams and status back to a traced real javac argv.
        raw_compile_stdout = case / (stem + "-compile.stdout")
        raw_compile_stderr = case / (stem + "-compile.stderr")
        matches = [c for c in field_cmds if Path(c["argv"][0]).name == "javac"
                   and f"/{family}/{stem}/" in " ".join(c["argv"])
                   and sha(c["stdout"]) == sha(raw_compile_stdout)
                   and sha(c["stderr"]) == sha(raw_compile_stderr)]
        require(bool(matches) and all(c["exit"] == expected_compile for c in matches),
                f"field compiler raw transcript/argv/exit {leg}/{family}/{version}")
        # The runner's report is JSON from the CLI; require the saved stdout bytes to match it.
        cli = read(case / (stem + ".json"))
        cli_stdout = (case / (stem + ".stdout")).read_text()
        require(json.loads(cli_stdout) == cli, f"field CLI stdout/report consistency {leg}/{family}/{version}")
        cli_bin = CANDIDATE if version == "candidate" else BASELINE
        cli_records = [c for c in field_cmds if Path(c["argv"][0]) == cli_bin
                       and len(c["argv"]) > 1 and c["argv"][1] == "class-source"
                       and "--class" in c["argv"] and c["argv"][c["argv"].index("--class") + 1] == family
                       and sha(c["stdout"]) == sha(case / (stem + ".stdout"))
                       and sha(c["stderr"]) == sha(case / (stem + ".stderr"))]
        require(bool(cli_records) and all(c["exit"] == 0 for c in cli_records),
                f"field CLI raw transcript/argv/exit {leg}/{family}/{version}")
        if expected_compile == 0:
            raw_out = lines(case / (stem + "-run.stdout"))
            raw_err = lines(case / (stem + "-run.stderr"))
            require(not raw_err, f"field runtime stderr {leg}/{family}/{version}")
            require(row.get("run") == 0, f"field runtime status {leg}/{family}/{version}")
            runs = [c for c in field_cmds if Path(c["argv"][0]).name == "java"
                    and "-Xverify:all" in c["argv"]
                    and f"/{family}/{stem}/classes" in " ".join(c["argv"])
                    and sha(c["stdout"]) == sha(case / (stem + "-run.stdout"))
                    and sha(c["stderr"]) == sha(case / (stem + "-run.stderr"))]
            require(bool(runs) and all(c["exit"] == row["run"] for c in runs),
                    f"field runtime raw transcript/argv/exit {leg}/{family}/{version}")
        else:
            raw_out = []
            require(family == "SCGB", f"unexpected field compile refusal {leg}/{family}/{version}")
            require("jarde_refused_body" in "\n".join(compile_err), f"field SCGB refusal diagnostic {leg}/{family}")
            require("jarde_refused_body();" in read(case / (stem + ".json"))["text"], f"field SCGB refusal source marker {leg}/{family}")
            require("behavior_match" not in row and "reflection_match" not in row, f"field refused summary shape {leg}/{family}")
        if expected_compile != 0:
            if version == "candidate":
                fcounts["candidate_expected_refused_compile"] += 1
            continue
        api = [x for x in raw_out if x.startswith(api_prefixes)]
        behavior = [x for x in raw_out if not x.startswith(api_prefixes)]
        # Recompute comparison against original input execution, and against baseline success.
        original = lines(case / "original-run.stdout")
        original_api = [x for x in original if x.startswith(api_prefixes)]
        original_behavior = [x for x in original if not x.startswith(api_prefixes)]
        api_match = api == original_api
        behavior_match = behavior == original_behavior
        fcounts[version + "_api_match"] += api_match
        fcounts[version + "_behavior_match"] += behavior_match
        require(row["behavior_match"] == behavior_match, f"field recomputed behavior summary {leg}/{family}/{version}")
        require(row["reflection_match"] == api_match, f"field recomputed reflection summary {leg}/{family}/{version}")
        if version == "candidate":
            baseline_api = [x for x in lines(case / "baseline-run.stdout") if x.startswith(api_prefixes)]
            if baseline_api == original_api and api != original_api:
                f_regressions.append(f"{leg}/{family}/api")
        fcounts["raw_compile_stream_bytes"] += len("\n".join(compile_out + compile_err).encode())
    # The known same-class generic consumer boundary is deliberately refused by CLI8.
    scgb2 = []
    for (leg, family, version), row in frows.items():
        if version != "candidate" or family != "SCGB":
            continue
        scgb2.append((leg, family))
        require(row["compile"] == 1, "field expected refusal status " + leg + "/" + family)
    require(len(scgb2) == 2, "field expected SCGB refusal scope count")
    metrics["field"] = {"matrix_rows": len(fsummary), "candidate_rows": sum(r["version"] == "candidate" for r in fsummary),
                        "actual_commands": len(field_cmds), "recomputed": dict(fcounts),
                        "expected_refusal_candidate_rows": len(scgb2), "accepted_api_regressions": f_regressions}

    # Explicitly gate the historical ListWrong recovery on both JDK legs while preserving raw put.
    listwrong = {}
    for leg in ("javac8", "javac23"):
        case = FIELD / "adapter-run/root" / leg / "ListWrong"
        original = lines(case / "original-run.stdout")
        candidate = lines(case / "candidate-run.stdout")
        source = (case / "candidate.java").read_text()
        expected_api = [x for x in original if x.startswith(api_prefixes)]
        candidate_api = [x for x in candidate if x.startswith(api_prefixes)]
        require(expected_api == candidate_api and "fieldGenericType=java.util.List<T>" in candidate_api,
                "CLI8 ListWrong field generic type restored " + leg)
        require("generic Signature projection refused for `put(Ljava/util/List;)V`" in source
                and "public void put(java.util.List arg1)" in source,
                "CLI8 ListWrong put remains explicitly raw " + leg)
        listwrong[leg] = {"original_field_api": expected_api, "candidate_field_api": candidate_api,
                          "put_raw_signature_refusal": True}
    metrics["ListWrong_cross_jdk"] = listwrong

    # Constructor preflight, frozen source/JAR inputs, tools and command transcripts.
    cp = read(CTOR / "preflight.json")
    cm = read(CTOR / "run-metadata.json")
    cs = read(CTOR / "summary.json")
    require(cp["candidate_cli"]["sha256"] == CAND_SHA and cp["baseline_cli"]["sha256"] == BASE_SHA, "constructor preflight CLI identity")
    require(cm["candidate_cli_sha256"] == CAND_SHA and cm["baseline_cli_sha256"] == BASE_SHA, "constructor metadata CLI identity")
    require(cm["failure"] is None, "constructor runner failure")
    require(sha(cm["preflight"]) == cm["preflight_sha256"], "constructor preflight hash")
    for key, digestkey, label in (("wrapper_snapshot", "wrapper_sha256", "constructor wrapper"),
                                  ("runner_snapshot", "runner_snapshot_sha256", "constructor runner snapshot")):
        same_hash(cp[key], cp[digestkey], label)
    same_hash(cm["runner_snapshot"], cm["runner_snapshot_sha256"], "constructor executed snapshot")
    for tool in cp["tools"]:
        same_hash(tool["path"], tool["sha256"], "constructor tool " + tool["label"])
    for v in cp["tool_version_commands"]:
        require(v["exit"] == 0 or (v["label"] == "corretto8_jar" and v["argv"][-1] == "-help" and v["exit"] == 1),
                "constructor version command " + v["label"])
    require(len(cp["frozen_inputs"]) == cp["frozen_input_count"] == 242, "constructor frozen input count")
    for row in cp["frozen_inputs"]:
        same_hash(row["path"], row["sha256"], "constructor frozen input")
    lineage = cp["source_lineage"]
    require(lineage["fixture_source_count"] == 20 and lineage["frozen_source_copy_count"] == 80, "constructor source lineage counts")
    same_hash(lineage["checksums_path"], lineage["checksums_sha256"], "constructor checksums")
    for row in lineage["fixture_sources"] + lineage["frozen_source_copies"]:
        same_hash(row["path"], row["sha256"], "constructor source lineage")
    ccommands = cm["commands"]
    for c in ccommands:
        for stream in ("stdout", "stderr"):
            same_hash(c[stream], c[stream + "_sha256"], "constructor command " + stream)
        argv = c["argv"]
        exe = Path(argv[0]).name
        if exe == "javac":
            require("-classpath" in argv and "-sourcepath" in argv, "constructor javac isolation argv")
            if "-classpath" in argv and "-sourcepath" in argv:
                require(argv[argv.index("-classpath") + 1] == cp["empty_classpath"], "constructor empty classpath")
                require(argv[argv.index("-sourcepath") + 1] == cp["empty_sourcepath"], "constructor empty sourcepath")
        elif exe == "java":
            cp_idx = next((i for i, x in enumerate(argv[:-1]) if x in ("-cp", "-classpath")), None)
            require(cp_idx is not None and "-Xverify:all" in argv, "constructor JVM verification argv")
            if cp_idx is not None:
                classpath = argv[cp_idx + 1]
                require(".jar" not in classpath.lower() and "/input/" not in classpath, "constructor runtime isolation")
    require(len(ccommands) > 0, "constructor actual command transcript missing")

    # Recompute constructor behavior and reflective API observations from raw run stdout.
    crows = {(r["leg"], r["debug"], r["class"], r["version"]): r for r in cs["jarde"]}
    require(len(cs["jarde"]) == 160 and len(crows) == 160, "constructor summary matrix cardinality")
    ccounts = Counter()
    c_regressions = []
    for (leg, debug, family, version), row in crows.items():
        case = CTOR / leg / debug / family
        side = "candidate" if version == "candidate" else "baseline"
        cli_bin = CANDIDATE if version == "candidate" else BASELINE
        cli_stdout = case / (side + "/class-source.stdout")
        cli_stderr = case / (side + "/class-source.stderr")
        cli_records = [c for c in ccommands if Path(c["argv"][0]) == cli_bin
                       and len(c["argv"]) > 1 and c["argv"][1] == "class-source"
                       and "--class" in c["argv"] and c["argv"][c["argv"].index("--class") + 1] == family
                       and sha(c["stdout"]) == sha(cli_stdout) and sha(c["stderr"]) == sha(cli_stderr)]
        require(bool(cli_records) and all(c["exit"] == row["cli_exit"] for c in cli_records),
                f"constructor CLI raw transcript/argv/exit {leg}/{debug}/{family}/{version}")
        expected = lines(case / "original-run.stdout")
        compile_exit = int((case / (side + "/javac.exit")).read_text().strip())
        require(row["compile_exit"] == compile_exit, f"constructor compile exit summary {leg}/{debug}/{family}/{version}")
        if compile_exit == 0:
            run_exit = int((case / (side + "/run.exit")).read_text().strip())
            require(row["run_exit"] == run_exit == 0, f"constructor run exit {leg}/{debug}/{family}/{version}")
            require(not lines(case / (side + "/run.stderr")), f"constructor runtime stderr {leg}/{debug}/{family}/{version}")
            actual = lines(case / (side + "/run.stdout"))
            behavior = [x for x in actual if x.startswith("BEHAVIOR|")]
            reflection = [x for x in actual if x.startswith("REFLECT|")]
            expected_behavior = [x for x in expected if x.startswith("BEHAVIOR|")]
            expected_reflection = [x for x in expected if x.startswith("REFLECT|")]
            bm, rm = behavior == expected_behavior, reflection == expected_reflection
        else:
            require(row["run_exit"] is None, f"constructor no run after failed compile {leg}/{debug}/{family}/{version}")
            actual, bm, rm = [], False, False
        ccounts[version + "_behavior_match"] += bm
        ccounts[version + "_reflection_match"] += rm
        require(row["behavior_match"] == bm, f"constructor behavior summary {leg}/{debug}/{family}/{version}")
        require(row["reflection_match"] == rm, f"constructor reflection summary {leg}/{debug}/{family}/{version}")
        if compile_exit == 0:
            require(row["run_output"].splitlines() == actual, f"constructor summary raw stdout {leg}/{debug}/{family}/{version}")
        if version == "candidate":
            baseline_compile = int((case / "baseline/javac.exit").read_text().strip())
            baseline_reflection = []
            if baseline_compile == 0:
                b = lines(case / "baseline/run.stdout")
                baseline_reflection = [x for x in b if x.startswith("REFLECT|")]
            if baseline_compile == 0 and baseline_reflection == [x for x in expected if x.startswith("REFLECT|")] and not rm:
                c_regressions.append(f"{leg}/{debug}/{family}/reflection")
    metrics["constructor"] = {"matrix_rows": len(cs["jarde"]), "candidate_rows": sum(r["version"] == "candidate" for r in cs["jarde"]),
                              "actual_commands": len(ccommands), "recomputed": dict(ccounts),
                              "accepted_api_regressions": c_regressions}

    result = {
        "scope": "Independent saved-evidence audit of GC09 field-23-v9 and constructor-80-v8, fixed CLI8 only; no Java/candidate reruns.",
        "passed": not errors and not f_regressions and not c_regressions,
        "cli": {"baseline_sha256": BASE_SHA, "candidate_sha256": CAND_SHA},
        "metrics": metrics,
        "confirmed_regressions": {"field_vs_baseline_accepted_api": f_regressions,
                                  "constructor_vs_baseline_accepted_api": c_regressions},
        "limitations": ["Results establish observed behavior for the fixed CLI8 artifact only.",
                        "They do not prove the constructor source regression has been repaired.",
                        "Field GC09 deliberately retains the SCGB producer consumer refusal boundary."],
        "errors": errors,
        "verifier_sha256": sha(Path(__file__)),
    }
    out = HERE / "gc09-field-constructor-v8-verification.json"
    out.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    raise SystemExit(0 if not errors else 1)


if __name__ == "__main__":
    main()
