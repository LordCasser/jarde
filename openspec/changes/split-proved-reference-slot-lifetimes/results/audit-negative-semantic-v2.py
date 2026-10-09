#!/usr/bin/env python3
"""Compare saved candidate-v2 negative reports with immutable baselines.
Only a JSON member whose key is exactly ``elapsed_millis`` is omitted.
This script performs no CLI/compiler invocation and writes one audit JSON.
"""
from __future__ import annotations
import hashlib, json, pathlib, tarfile

HERE = pathlib.Path(__file__).resolve().parent
CHANGE = HERE.parent
CAND_ROOT = HERE / "candidate-v2"
NEG_BASELINE = CHANGE / "evidence" / "negative-baseline-20261009.tar.gz"
NEG_ROOT = "jarde-ref-ref-slot-negative-20261009"
OUT = HERE / "negative-semantic-audit-v2.json"
NONZERO_CAND = HERE / "nonzero-held-candidate-v2"
NONZERO_BASE = HERE / "nonzero-held-baseline-v2"


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def sha_file(path: pathlib.Path) -> str:
    return sha_bytes(path.read_bytes())

def canonical_hash(value) -> str:
    return sha_bytes(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode())

def load_json_bytes(data: bytes):
    return json.loads(data.decode("utf-8"))

def compare(a, b, path=""):
    """Return leaf-level JSON diffs, omitting only elapsed_millis fields."""
    if isinstance(a, dict) and isinstance(b, dict):
        out=[]
        for k in sorted(set(a)|set(b)):
            if k == "elapsed_millis":
                continue
            p=f"{path}/{k}"
            if k not in a: out.append({"path":p,"baseline":"<missing>","candidate":b[k]})
            elif k not in b: out.append({"path":p,"baseline":a[k],"candidate":"<missing>"})
            else: out.extend(compare(a[k],b[k],p))
        return out
    if isinstance(a,list) and isinstance(b,list):
        out=[]
        for i in range(max(len(a),len(b))):
            p=f"{path}/{i}"
            if i>=len(a): out.append({"path":p,"baseline":"<missing>","candidate":b[i]})
            elif i>=len(b): out.append({"path":p,"baseline":a[i],"candidate":"<missing>"})
            else: out.extend(compare(a[i],b[i],p))
        return out
    return [] if a == b else [{"path":path or "/","baseline":a,"candidate":b}]

def source_maps(report):
    return [m.get("outcome",{}).get("report",{}).get("source_map") for m in report.get("methods",[])]

def text_bytes(report):
    return report["text"].encode("utf-8")

def location_anchor_projection(report):
    """Capture source spans and instruction anchors separately from owner provenance."""
    result=[]
    for method in report.get("methods",[]):
        source_map=method.get("outcome",{}).get("report",{}).get("source_map") or {}
        segments=[]
        for segment in source_map.get("segments",[]):
            origin=segment.get("origin",{})
            primary=origin.get("primary") or {}
            derived=origin.get("derived") or []
            def anchor(item):
                mi=item.get("method",{})
                return {"bci":item.get("bci"),"cp":item.get("cp"),"provenance":item.get("provenance"),"method_name":mi.get("name"),"method_descriptor":mi.get("descriptor")}
            segments.append({"start":segment.get("start"),"end":segment.get("end"),"primary":anchor(primary),"derived":[anchor(item) for item in derived]})
        result.append({"declaration":method.get("declaration"),"segments":segments})
    return result

def snapshot_identities(value):
    found=set()
    def visit(node):
        if isinstance(node,dict):
            for key,val in node.items():
                if key=="snapshot" and isinstance(val,str): found.add(val)
                visit(val)
        elif isinstance(node,list):
            for val in node: visit(val)
    visit(value)
    return sorted(found)

def metadata_summary(report):
    out=[]
    for m in report.get("methods",[]):
        r=m.get("outcome",{}).get("report",{})
        out.append({"declaration":m.get("declaration"),"aliased_names":r.get("aliased_names"),"diagnostics":r.get("diagnostics"),"markers":m.get("markers")})
    return out

def read_tar_json(tf, name):
    f=tf.extractfile(name)
    if f is None: raise FileNotFoundError(name)
    raw=f.read()
    return raw,load_json_bytes(raw)

def command_for(commands,label):
    matches=[c for c in commands if c.get("label")==label]
    if len(matches)!=1: raise ValueError(f"expected one command {label}, got {len(matches)}")
    return matches[0]

cand_manifest_path=CAND_ROOT/"manifest.json"
cand_manifest=json.loads(cand_manifest_path.read_text())
neg_archive_meta=cand_manifest["negative_archive"]
archive_sha=sha_file(CHANGE/"evidence"/neg_archive_meta["archive"])
if archive_sha != neg_archive_meta["sha256"]: raise SystemExit("negative archive SHA mismatch")
comparisons=[]
with tarfile.open(CHANGE/"evidence"/neg_archive_meta["archive"],"r:gz") as tf:
    baseline_manifest_path=f"{NEG_ROOT}/replay-cli9-v3/manifest.json"
    bm_raw,bm=read_tar_json(tf,baseline_manifest_path)
    for row in cand_manifest["negative_rows"]:
        family=row["family"]
        # Select the command by its recorded stdout, not the repeated generic label.
        cc_matches=[c for c in cand_manifest["commands"] if c.get("stdout")==f"negative/{family}/class-source.stdout"]
        if len(cc_matches)!=1: raise ValueError(f"candidate command mismatch: {family}")
        cc=cc_matches[0]
        candidate_path=CAND_ROOT/cc["stdout"]
        candidate_raw=candidate_path.read_bytes(); candidate=load_json_bytes(candidate_raw)
        if sha_bytes(candidate_raw) != cc["stdout_sha256"]: raise SystemExit(f"candidate stdout hash mismatch {family}")
        bc=command_for(bm["commands"],f"cli9-class-source-{family}")
        baseline_stdout=bc["stdout"].replace("/private/tmp/jarde-ref-ref-slot-negative-20261009/",f"{NEG_ROOT}/")
        baseline_raw,baseline=read_tar_json(tf,baseline_stdout)
        if sha_bytes(baseline_raw) != bc["stdout_sha256"]: raise SystemExit(f"baseline stdout hash mismatch {family}")
        baseline_input_path=bc["argv"][bc["argv"].index("--input")+1].replace("/private/tmp/jarde-ref-ref-slot-negative-20261009/",f"{NEG_ROOT}/")
        baseline_jar_raw=tf.extractfile(baseline_input_path).read()
        if sha_bytes(baseline_jar_raw) != bc["jar_sha256"]: raise SystemExit(f"baseline input jar hash mismatch {family}")
        diffs=compare(baseline,candidate)
        bmaps=source_maps(baseline); cmaps=source_maps(candidate)
        bmeta=metadata_summary(baseline); cmeta=metadata_summary(candidate)
        # Resolve recorded jar input from argv to avoid inventing family filename casing.
        argv=cc["argv"]; input_path=pathlib.Path(argv[argv.index("--input")+1])
        input_sha=sha_file(input_path)
        base_input_sha=bc["jar_sha256"]
        if input_sha != row["input_sha256"]: raise SystemExit(f"candidate input hash mismatch {family}")
        rendered_candidate_sha=sha_bytes(text_bytes(candidate))
        if rendered_candidate_sha != row["source_sha256"]: raise SystemExit(f"candidate rendered-source hash mismatch {family}")
        original_class=bc["argv"][bc["argv"].index("--class")+1]
        original_source_path=f"{NEG_ROOT}/replay-cli9-v3/{family}/{original_class}.java"
        original_source_raw=tf.extractfile(original_source_path).read()
        if sha_bytes(original_source_raw) != bc.get("source_sha256"): raise SystemExit(f"baseline original-source hash mismatch {family}")
        comparisons.append({
            "family":family,"candidate":{"report_path":str(candidate_path.relative_to(HERE)),"report_sha256":sha_bytes(candidate_raw),"input_path":str(input_path),"input_sha256":input_sha,"source_sha256":row["source_sha256"],"exit":cc["exit"]},
            "baseline":{"archive_member":baseline_stdout,"report_sha256":sha_bytes(baseline_raw),"input_archive_member":baseline_input_path,"input_sha256":sha_bytes(baseline_jar_raw),"source_sha256":bc.get("source_sha256"),"exit":bc["exit"]},
            "rendered_text_bytes":{"baseline_sha256":sha_bytes(text_bytes(baseline)),"candidate_sha256":rendered_candidate_sha,"equal":text_bytes(baseline)==text_bytes(candidate)},
            "source_sha256_semantics":{"candidate_manifest":"rendered report text UTF-8 bytes (verified against this report)","baseline_manifest":"original Java fixture bytes (verified against archived source file)","cross_value_equality_is_not_a_rendered-source-comparison":True},
            "source_map":{"baseline_sha256":canonical_hash(bmaps),"candidate_sha256":canonical_hash(cmaps),"equal":bmaps==cmaps},
            "held_use_metadata":{"baseline":bmeta,"candidate":cmeta,"equal":bmeta==cmeta} if family=="held-use" else None,
            "diff_count_after_elapsed_millis_only":len(diffs),"diffs":diffs
        })

nonzero=[]
for leg in ("javac8","javac23"):
    bp=NONZERO_BASE/leg/"report.stdout"; cp=NONZERO_CAND/leg/"report.stdout"
    braw=bp.read_bytes(); craw=cp.read_bytes(); b=load_json_bytes(braw); c=load_json_bytes(craw)
    bmj=json.loads((NONZERO_BASE/"manifest.json").read_text()); cmj=json.loads((NONZERO_CAND/"manifest.json").read_text())
    br=next(x for x in bmj["rows"] if x["leg"]==leg); cr=next(x for x in cmj["rows"] if x["leg"]==leg)
    bc=command_for(bmj["commands"],f"{leg}/report"); cc=command_for(cmj["commands"],f"{leg}/report")
    if sha_bytes(braw) != bc["stdout_sha256"] or sha_bytes(craw) != cc["stdout_sha256"]: raise SystemExit(f"nonzero stdout hash mismatch {leg}")
    bjar=pathlib.Path(bc["argv"][bc["argv"].index("--input")+1]); cjar=pathlib.Path(cc["argv"][cc["argv"].index("--input")+1])
    if sha_file(bjar) != br["jar_sha256"] or sha_file(cjar) != cr["jar_sha256"]: raise SystemExit(f"nonzero input jar hash mismatch {leg}")
    bmaps=source_maps(b); cmaps=source_maps(c)
    bprojection=location_anchor_projection(b); cprojection=location_anchor_projection(c)
    nonzero.append({
      "leg":leg,"baseline":{"report_path":str(bp.relative_to(HERE)),"report_sha256":sha_bytes(braw),"input_jar_path":next(iter([bc["argv"][bc["argv"].index("--input")+1]]),None) if "--input" in bc["argv"] else None,"input_jar_sha256":sha_file(bjar),"class_sha256":br["class_sha256"],"source_sha256":br["source_sha256"]},
      "candidate":{"report_path":str(cp.relative_to(HERE)),"report_sha256":sha_bytes(craw),"input_jar_path":next(iter([cc["argv"][cc["argv"].index("--input")+1]]),None) if "--input" in cc["argv"] else None,"input_jar_sha256":sha_file(cjar),"class_sha256":cr["class_sha256"],"source_sha256":cr["source_sha256"]},
      "rendered_text_bytes":{"baseline_sha256":sha_bytes(text_bytes(b)),"candidate_sha256":sha_bytes(text_bytes(c)),"equal":text_bytes(b)==text_bytes(c)},
      "source_map":{"baseline_sha256":canonical_hash(bmaps),"candidate_sha256":canonical_hash(cmaps),"equal":bmaps==cmaps,"locations_and_instruction_anchors":{"baseline":bprojection,"candidate":cprojection,"equal":bprojection==cprojection},"provenance_snapshot_identities":{"baseline":snapshot_identities(bmaps),"candidate":snapshot_identities(cmaps),"equal":snapshot_identities(bmaps)==snapshot_identities(cmaps)}},
      "held_use_metadata":{"baseline":metadata_summary(b),"candidate":metadata_summary(c),"equal":metadata_summary(b)==metadata_summary(c)},
      "diff_count_after_elapsed_millis_only":len(compare(b,c)),"diffs":compare(b,c)
    })

result={
 "schema":"negative-semantic-audit-v2",
 "method":{"comparison":"recursive JSON leaf comparison","only_omitted_key":"elapsed_millis (exact key, at any depth)","inputs":"saved raw JSON outputs; no CLI/compiler invoked","source_sha256_field_semantics":"candidate negative_rows hashes rendered report text; archived baseline command source_sha256 hashes the original Java fixture, verified from the corresponding archived .java bytes","nonzero_source_map":"full maps are compared strictly; location/anchor projection and snapshot provenance identities are reported separately, with no normalization applied to the actual diff"},
 "audit_script":{"path":str(pathlib.Path(__file__).relative_to(CHANGE)),"sha256":sha_file(pathlib.Path(__file__))},
 "candidate_manifest":{"path":str(cand_manifest_path.relative_to(CHANGE)),"sha256":sha_file(cand_manifest_path),"runner_sha256":cand_manifest["runner_sha256"],"cli":cand_manifest["cli"]},
 "negative_baseline":{"archive":str((CHANGE/"evidence"/neg_archive_meta["archive"]).relative_to(CHANGE)),"sha256":archive_sha,"manifest_member":baseline_manifest_path,"manifest_sha256":sha_bytes(bm_raw),"cli_sha256":bm["cli_sha256_pre"],"runner":bm["executed_runner"]},
 "negative_reports":comparisons,
 "nonzero_held":{"candidate_manifest_sha256":sha_file(NONZERO_CAND/"manifest.json"),"baseline_manifest_sha256":sha_file(NONZERO_BASE/"manifest.json"),"candidate_runner_sha256":cmj["runner_sha256"],"baseline_runner_sha256":bmj["runner_sha256"],"candidate_cli":cmj["cli"],"candidate_cli_sha256":cmj["cli_sha256"],"baseline_cli":bmj["cli"],"baseline_cli_sha256":bmj["cli_sha256"],"reports":nonzero}
}
OUT.write_text(json.dumps(result,indent=2,ensure_ascii=False)+"\n")
print(OUT)
print("negative report diffs:", {x["family"]:x["diff_count_after_elapsed_millis_only"] for x in comparisons})
print("nonzero held diffs:", {x["leg"]:x["diff_count_after_elapsed_millis_only"] for x in nonzero})
