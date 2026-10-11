#!/usr/bin/env python3
import hashlib,json,re,sys
from pathlib import Path
ROOT=Path("/private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3")
FACTS=Path("/private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3-facts/facts.json")
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
d=json.loads((ROOT/"execution.json").read_text()); f=json.loads(FACTS.read_text())
need(d["status"]=="passed_SSA_diagnostic_only","status")
need(d["base_region_sha256"]=="acdb5d22195ce0b5d283fe1b7a3003f35830c85811b18e2e713b7f4728e93168","base SHA")
need(d["candidate_region_sha256"]=="c368ba49d7f5d922c1bde4513fe391d54de28d615d58d7637fe6f6e11cb7b188","candidate SHA")
need(d["patch_sha256"]=="3c3f6ae3d20b9e64e64f2511bde69b407084dd9c456e6a163eff98bd38a19e32","patch SHA")
candidate_dir=Path("/private/tmp/jarde-computed-for-real-ir-diagnostic-luna-v1")
need(sha(candidate_dir/"base-region.rs")==d["base_region_sha256"],"actual base artifact SHA")
need(sha(candidate_dir/"candidate-region.rs")==d["candidate_region_sha256"],"actual candidate artifact SHA")
need(sha(candidate_dir/"diagnostic.patch")==d["patch_sha256"],"actual patch artifact SHA")
need(d["source_pins_before"]==d["source_pins_after_restore"],"all source pins restored")
need(len(d["source_pins_before"])==132,"source pin count")
changed=[k for k in d["source_pins_before"] if d["source_pins_before"][k]!=d["diagnostic_source_pins"].get(k)]
need(changed==["crates/jarde-java/src/region.rs"],"temporary change limited to region.rs")
need(d["source_pins_before"][changed[0]]==d["base_region_sha256"],"base source pin")
need(d["diagnostic_source_pins"][changed[0]]==d["candidate_region_sha256"],"temporary candidate source pin")
need(d["restored"] is True and d["target_absent"] is True,"restored/target absent")
need(len(d["commands"])==3,"command record count")
need(d["commands"][0]["argv"][:3]==["git","apply","--check"] and d["commands"][0]["exit_code"]==0,"patch check command")
need(d["commands"][1]["argv"][:2]==["git","apply"] and d["commands"][1]["exit_code"]==0,"temporary patch apply")
for idx in (0,1):
    c_apply=d["commands"][idx]
    for stream in ("stdout","stderr"):
        raw=ROOT/(f"apply-{idx}.{stream}.raw")
        need(sha(raw)==c_apply[stream+"_sha256"],f"apply {idx} {stream} raw SHA")
c=d["commands"][2]; argv=c["argv"]
need(argv==["cargo","test","-p","jarde-java","--lib","--locked","region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir","--","--exact","--nocapture"],"exact test argv")
need(c["exit_code"]==0 and c["guard_stop"] is None,"exact test success/guard")
for stream in ("stdout","stderr"):
    meta=c["streams"][stream]; p=Path(meta["path"]); b=p.read_bytes()
    need(len(b)==meta["bytes"] and hashlib.sha256(b).hexdigest()==meta["sha256"],stream+" raw hash/length")
stdout=(ROOT/"0.stdout.raw").read_text(); stderr=(ROOT/"0.stderr.raw").read_text()
need("running 1 test" in stdout and "1 passed; 0 failed; 0 ignored; 0 measured; 350 filtered out" in stdout,"one-test summary")
need("CF07_REAL_IR_DIAG class_bytes=419 class_blake3=52ddcc0b5ab5de0702266901380266b53498433df87ca02f4861c61c7f4c9b19" in stderr,"frozen class marker")
init_rows=re.findall(r"CF07_INIT_INSN .*?instruction=SsaInstruction \{\n\s*bci: (\d+),\n\s*opcode: (\d+),",stderr,re.S)
need([(int(b),int(op)) for b,op in init_rows]==[(0,29),(1,4),(2,100),(3,54)],"init BCI/opcode sequence")
slot_rows=[(int(b),int(v)) for b,v in re.findall(r"^CF07_SLOT4_VALUE bci=(\d+) slot=Local\(4\) value_id=ValueId\((\d+)\)",stderr,re.M)]
need(slot_rows==[(3,12),(5,8),(12,8),(19,8),(22,8),(22,21)],"all local-4 access values")
expected={"CF07_INIT_INSN":4,"CF07_INIT_VALUE":8,"CF07_RELEVANT_PHI":5,"CF07_SLOT4_ACCESS":5,"CF07_NATURAL_LOOP":1,"CF07_RETURN_LEAVES":1,"CF07_UPDATE_LATCH":1,"CF07_UPDATE_EDGE":2}
actual={k:len(re.findall(r"^"+re.escape(k)+r"\b",stderr,re.M)) for k in expected}
need(actual==expected and actual==d["diagnostic_marker_counts"],"marker counts")
for fragment in ["value_id=ValueId(11)","bci: 2,","bci: Some(\n                3,\n            )","value_id=ValueId(12)","CF07_RETURN_LEAVES nodes={4} normal_exit=Some(3)"]:
    need(fragment in stderr, "diagnostic fact: "+fragment)
need(re.search(r"slot: Local\(\s*4,\s*\)",stderr) is not None,"header phi is local 4")
need(re.search(r"value: ValueId\(\s*8,\s*\)",stderr) is not None,"header phi value id 8")
phi_start=stderr.rfind("CF07_RELEVANT_PHI SsaPhi {")
phi_end=stderr.index("CF07_SLOT4_ACCESS",phi_start)
phi_section=stderr[phi_start:phi_end]
need("slot: Local(\n        4," in phi_section and "value: ValueId(\n        8," in phi_section,"header phi slot/id")
need(phi_section.index("ValueId(\n                12,") < phi_section.index("ValueId(\n                21,"),"header phi inputs 12 then 21")
need(all(f"bci: Some(\n                {use}," in phi_section for use in (5,12,19,22)),"header phi use BCIs")
need("blocks: {\n        1,\n        2,\n        5," in stderr and "latches: {\n        5," in stderr,"natural loop/latch IDs")
edges=re.findall(r"CF07_UPDATE_EDGE CanonicalEdge \{(.*?)\n\}",stderr,re.S)
edge_pairs=[]
for edge in edges:
    m=re.search(r"from: CanonicalBlockId \{\s*bci: (\d+),.*?to: CanonicalBlockId \{\s*bci: (\d+),",edge,re.S)
    if m: edge_pairs.append((int(m.group(1)),int(m.group(2))))
need(edge_pairs==[(11,22),(22,5)],"update/goto edge endpoints")
return_edges=re.findall(r"CF07_RETURN_LEAF_EDGE CanonicalEdge \{(.*?)\n\}",stderr,re.S)
need(len(return_edges)==1 and re.search(r"from: CanonicalBlockId \{\s*bci: 11,.*?to: CanonicalBlockId \{\s*bci: 19,",return_edges[0],re.S) is not None,"return leaf's single incoming edge")
need("CF07_RETURN_LEAVES nodes={4} normal_exit=Some(3)" in stderr and "root_frame=Frame { boundary: None" in stderr,"return certificate/frame context")
need("bci: 21,\n            opcode: 172" in stderr,"return leaf ends at ireturn BCI 21")
clean=d["clean_command"]; need(clean["argv"]==["cargo","clean"] and clean["exit_code"]==0 and clean["guard_stop"] is None,"cargo clean record")
for stream in ("stdout","stderr"):
    m=clean["streams"][stream]; b=Path(m["path"]).read_bytes(); need(len(b)==m["bytes"] and hashlib.sha256(b).hexdigest()==m["sha256"],"clean "+stream+" raw")
print("PASS: root v3 SSA diagnostic record/raw/facts independently verified; Builder statement evidence remains outstanding")
