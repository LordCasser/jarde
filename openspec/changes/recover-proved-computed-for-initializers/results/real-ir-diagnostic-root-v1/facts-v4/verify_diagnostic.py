#!/usr/bin/env python3
"""Read-only verifier for root v3's captured CF07 SSA diagnostic."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path("/private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3")
FACT_DIR = Path("/private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3-facts-v2")
FACTS = json.loads((FACT_DIR / "facts.json").read_text())
RECORD = json.loads((ROOT / "execution.json").read_text())
STDOUT = (ROOT / "0.stdout.raw").read_text()
STDERR = (ROOT / "0.stderr.raw").read_text()


def need(condition, message):
    if not condition:
        raise SystemExit("FAIL: " + message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def uses_from(value_text):
    start = value_text.find("uses:")
    end = value_text.find("replaced_by:", start)
    need(start >= 0 and end > start, "SSA value use section")
    section = value_text[start:end]
    found = []
    pattern = re.compile(
        r"SsaUse\s*\{\s*block:\s*CanonicalBlockId\s*\{\s*"
        r"bci:\s*(\d+),\s*path:\s*\[\],\s*\},\s*"
        r"bci:\s*(?:Some\(\s*(\d+),?\s*\)|None)", re.S
    )
    for match in pattern.finditer(section):
        found.append({"block_bci": int(match.group(1)), "bci": int(match.group(2)) if match.group(2) else None})
    return found


def fact_uses_match(raw_uses, expected_uses, value_id):
    """Compare typed use coordinates; permit only the documented phi-input annotation."""
    projected = []
    saw_phi_annotation = False
    for use in expected_uses:
        need(isinstance(use, dict), f"ValueId({value_id}) use must be an object")
        keys = set(use)
        unknown_keys = sorted(keys - {"block_bci", "bci", "as"})
        need(not unknown_keys, f"ValueId({value_id}) has unknown use annotation keys: {unknown_keys}")
        need({"block_bci", "bci"} <= keys, f"ValueId({value_id}) use lacks typed coordinates")
        need(isinstance(use["block_bci"], int) and (use["bci"] is None or isinstance(use["bci"], int)), f"ValueId({value_id}) use coordinate types")
        projected.append({"block_bci": use["block_bci"], "bci": use["bci"]})
        if "as" in use:
            need(use["as"] == "phi input", f"ValueId({value_id}) has unsupported use annotation")
            need(use["bci"] is None, f"ValueId({value_id}) phi input must have no instruction BCI")
            phi = FACTS["header_phi"]
            need(use["block_bci"] == phi["block_bci"], f"ValueId({value_id}) annotation is not at its recorded phi block")
            need(any(item["value"] == f"ValueId({value_id})" for item in phi["inputs"]), f"ValueId({value_id}) annotation lacks matching phi input")
            saw_phi_annotation = True
    need(raw_uses == projected, f"ValueId({value_id}) typed use coordinates bound to raw")
    # Each annotated use is now separately tied to the exact input value in the frozen header phi.
    if saw_phi_annotation:
        need(any(item["value"] == f"ValueId({value_id})" for item in FACTS["header_phi"]["inputs"]), f"ValueId({value_id}) phi annotation target")


def value_section(value_id, bci, slot, marker_kind="CF07_INIT_VALUE"):
    marker = f"{marker_kind} bci={bci} access_slot={slot} value_id=ValueId({value_id})"
    start = STDERR.find(marker)
    need(start >= 0, f"raw SSA value marker {marker}")
    end = STDERR.find("\nCF07_", start + len(marker))
    if end < 0:
        end = len(STDERR)
    return STDERR[start:end]


def def_from(value_text):
    start = value_text.find("def:")
    end = value_text.find("origin:", start)
    need(start >= 0 and end > start, "SSA def section")
    section = value_text[start:end]
    kind_match = re.search(r"def:\s*(Entry|Instruction|Phi)\s*\{", section)
    need(kind_match is not None, "SSA def kind")
    block = re.search(r"block:\s*CanonicalBlockId\s*\{\s*bci:\s*(\d+),", section)
    need(block is not None, "SSA def block")
    out = {"kind": kind_match.group(1), "block_bci": int(block.group(1))}
    if out["kind"] == "Instruction":
        insn_bci = re.search(r"\bbci:\s*(\d+),", section[block.end():])
        need(insn_bci is not None, "SSA instruction definition BCI")
        out["bci"] = int(insn_bci.group(1))
    if out["kind"] in ("Entry", "Phi"):
        slot = re.search(r"slot:\s*Local\(\s*(\d+),\s*\)", section)
        if slot:
            out["slot"] = f"Local({slot.group(1)})"
    return out


need(RECORD["status"] == "passed_SSA_diagnostic_only", "diagnostic-only record status")
need(RECORD["runner_sha256"] == FACTS["runner_sha256"], "runner SHA binding")
need(RECORD["base_region_sha256"] == RECORD["source_pins_before"]["crates/jarde-java/src/region.rs"], "base region pin")
need(RECORD["candidate_region_sha256"] == RECORD["diagnostic_source_pins"]["crates/jarde-java/src/region.rs"], "temporary candidate region pin")
need(RECORD["patch_sha256"] == FACTS["temporary_candidate"]["patch_sha256"], "patch SHA binding")
need(len(RECORD["source_pins_before"]) == 132, "source pin count")
need(RECORD["source_pins_before"] == RECORD["source_pins_after_restore"], "all source pins restored")
changed = [name for name, digest in RECORD["source_pins_before"].items() if RECORD["diagnostic_source_pins"].get(name) != digest]
need(changed == ["crates/jarde-java/src/region.rs"], "temporary source change limited to region.rs")
need(RECORD["restored"] is True and RECORD["target_absent"] is True, "restore/clean target state")
source_fact = FACTS["source_state"]
need(source_fact["before_region_sha256"] == RECORD["source_pins_before"]["crates/jarde-java/src/region.rs"], "facts base source pin")
need(source_fact["during_region_sha256"] == RECORD["candidate_region_sha256"], "facts temporary source pin")
need(source_fact["after_region_sha256"] == RECORD["source_pins_after_restore"]["crates/jarde-java/src/region.rs"], "facts restored source pin")
need(source_fact["before_after_all_132_pins_equal"] is True and source_fact["only_temporary_pin_difference"] == changed, "facts source restoration summary")
need(FACTS["temporary_candidate"] == {"base_sha256": RECORD["base_region_sha256"], "candidate_sha256": RECORD["candidate_region_sha256"], "patch_sha256": RECORD["patch_sha256"]}, "facts temporary candidate hashes")

candidate_dir = Path("/private/tmp/jarde-computed-for-real-ir-diagnostic-luna-v1")
for filename, key in (("base-region.rs", "base_region_sha256"), ("candidate-region.rs", "candidate_region_sha256"), ("diagnostic.patch", "patch_sha256")):
    need(sha(candidate_dir / filename) == RECORD[key], f"actual candidate artifact {filename} SHA")

need(len(RECORD["commands"]) == 3, "command record count")
need(RECORD["commands"][0]["argv"][:3] == ["git", "apply", "--check"] and RECORD["commands"][0]["exit_code"] == 0, "patch check record")
need(RECORD["commands"][1]["argv"][:2] == ["git", "apply"] and RECORD["commands"][1]["exit_code"] == 0, "temporary patch application record")
for index in (0, 1):
    for stream in ("stdout", "stderr"):
        need(sha(ROOT / f"apply-{index}.{stream}.raw") == RECORD["commands"][index][stream + "_sha256"], f"apply {index} {stream} raw SHA")
expected_argv = ["cargo", "test", "-p", "jarde-java", "--lib", "--locked", "region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir", "--", "--exact", "--nocapture"]
command = RECORD["commands"][2]
need(command["argv"] == expected_argv and FACTS["test"]["argv"] == expected_argv, "exact unit-test argv")
need(command["exit_code"] == 0 and command["guard_stop"] is None, "unit-test exit/guard")
need(FACTS["test"]["exit_code"] == command["exit_code"], "test fact exit code")
for stream in ("stdout", "stderr"):
    meta = command["streams"][stream]
    raw = Path(meta["path"]).read_bytes()
    need(len(raw) == meta["bytes"] == FACTS["test"][stream + "_bytes"], f"{stream} raw byte count")
    digest = hashlib.sha256(raw).hexdigest()
    need(digest == meta["sha256"] == FACTS["test"][stream + "_sha256"], f"{stream} raw SHA")
need("running 1 test" in STDOUT and "1 passed; 0 failed; 0 ignored; 0 measured; 350 filtered out" in STDOUT, "exact one-test summary")

# Compare facts.json's complete initial read/write rows to the raw SsaInstruction markers.
init = FACTS["ssa_init"]
raw_insns = [(int(bci), int(opcode)) for bci, opcode in re.findall(r"CF07_INIT_INSN .*?instruction=SsaInstruction \{\n\s*bci: (\d+),\n\s*opcode: (\d+),", STDERR, re.S)]
need(raw_insns == [(entry["bci"], entry["opcode"]) for entry in init], "initializer BCI/opcode facts bound to raw")
need([entry["instruction"] for entry in init] == ["iload_3", "iconst_1", "isub", "istore 4"], "initializer names correspond to recorded opcodes")
need(dict(zip((entry["opcode"] for entry in init), (entry["instruction"] for entry in init))) == {29: "iload_3", 4: "iconst_1", 100: "isub", 54: "istore 4"}, "initializer name/opcode mapping")
expected_accesses = []
for entry in init:
    expected_accesses.extend((entry["bci"], access["slot"], access["value"]) for access in entry["reads"])
    expected_accesses.extend((entry["bci"], access["slot"], access["value"]) for access in entry["writes"])
raw_accesses = [(int(bci), slot, f"ValueId({value})") for bci, slot, value in re.findall(r"^CF07_INIT_VALUE bci=(\d+) access_slot=([^ ]+) value_id=ValueId\((\d+)\)", STDERR, re.M)]
need(raw_accesses == expected_accesses, "initializer slot reads/writes bound to raw")
for entry, value_key in zip(init, ("value9", "value10", "value11", "value12")):
    write = entry["writes"][-1]
    value_id = int(re.search(r"ValueId\((\d+)\)", write["value"]).group(1))
    raw_value = value_section(value_id, entry["bci"], write["slot"])
    need(def_from(raw_value) == entry[value_key]["def"], f"{value_key} definition bound to raw")
    fact_uses_match(uses_from(raw_value), entry[value_key]["uses"], value_id)
source_value = value_section(3, 0, "Local(3)")
need(def_from(source_value) == init[0]["source_value3"]["def"], "entry argument ValueId(3) definition bound to raw")
need(uses_from(source_value) == init[0]["source_value3"]["uses"], "entry argument ValueId(3) uses bound to raw")

# Bind the exact header phi (slot, value, both inputs and every use) to the raw phi and SsaValue.
phi_fact = FACTS["header_phi"]
phi_marker = "CF07_RELEVANT_PHI SsaPhi {"
phi_start = STDERR.rfind(phi_marker)
phi_end = STDERR.index("CF07_SLOT4_ACCESS", phi_start)
phi_text = STDERR[phi_start:phi_end]
need(re.search(r"block: CanonicalBlockId \{\s*bci: 5,", phi_text) is not None, "header phi block")
need(re.search(r"slot: Local\(\s*4,\s*\)", phi_text) is not None, "header phi slot")
need(re.search(r"value: ValueId\(\s*8,\s*\)", phi_text) is not None, "header phi value id")
input_start = phi_text.find("inputs:")
value_start = phi_text.find("value=SsaValue", input_start)
need(input_start >= 0 and value_start > input_start, "header phi input raw section bounds")
input_close = phi_text.rfind("]", input_start, value_start)
need(input_close > input_start, "header phi input list closing bracket")
input_section = phi_text[input_start:input_close]
raw_inputs = [f"ValueId({value})" for value in re.findall(r"ValueId\(\s*(\d+),\s*\)", input_section)]
need(raw_inputs == [entry["value"] for entry in phi_fact["inputs"]], "header phi inputs in order")
phi_value = value_section(8, 5, "Local(4)", "CF07_SLOT4_VALUE")
need(def_from(phi_value) == phi_fact["definition"], "header phi definition bound to raw")
fact_uses_match(uses_from(phi_value), phi_fact["uses"], 8)

# Bind every local-4 read/write and the update's instruction definition/use.
expected_slot4 = []
for access in FACTS["local4_accesses"]:
    if "read" in access:
        expected_slot4.append((access["bci"], access["read"]))
    if "write" in access:
        expected_slot4.append((access["bci"], access["write"]))
raw_slot4 = [(int(bci), f"ValueId({value})") for bci, value in re.findall(r"^CF07_SLOT4_VALUE bci=(\d+) slot=Local\(4\) value_id=ValueId\((\d+)\)", STDERR, re.M)]
need(raw_slot4 == expected_slot4, "all slot-4 reads/writes bound to raw")
update = FACTS["update_value"]
update_text = value_section(21, 22, "Local(4)", "CF07_SLOT4_VALUE")
need(def_from(update_text) == update["value21_def"], "update definition bound to raw")
fact_uses_match(uses_from(update_text), [update["value21_use"]], 21)

# Bind loop node IDs/BCIs, return leaf/frame, and canonical edge endpoints.
loop_fact = FACTS["natural_loop"]
loop_start = STDERR.index("CF07_NATURAL_LOOP NaturalLoop {")
loop_end = STDERR.index("CF07_LOOP_BLOCK", loop_start)
loop_text = STDERR[loop_start:loop_end]
node_set = re.search(r"blocks: \{([^}]*)\}", loop_text, re.S)
latch_set = re.search(r"latches: \{([^}]*)\}", loop_text, re.S)
need(re.search(r"header: (\d+),", loop_text).group(1) == str(loop_fact["header_node"]), "natural loop header node")
need(node_set is not None and [int(x) for x in re.findall(r"\d+", node_set.group(1))] == loop_fact["block_nodes"], "natural loop block IDs")
need(latch_set is not None and [int(x) for x in re.findall(r"\d+", latch_set.group(1))] == loop_fact["latch_nodes"], "natural loop latch IDs")
raw_loop_bcis = [int(x) for x in re.findall(r"^CF07_LOOP_BLOCK node=\d+ id=CanonicalBlockId \{ bci: (\d+),", STDERR, re.M)]
need(raw_loop_bcis == loop_fact["block_bcis"], "natural loop BCI mapping")
need(re.search(r"irreducible: (true|false)", loop_text).group(1) == str(loop_fact["irreducible"]).lower(), "natural loop reducibility")
leaf_fact = FACTS["return_leaf"]
leaf_header = re.search(r"^CF07_RETURN_LEAF node=(\d+) id=CanonicalBlockId \{ bci: (\d+),", STDERR, re.M)
need(leaf_header is not None and tuple(map(int, leaf_header.groups())) == (leaf_fact["incoming_node"], leaf_fact["block_bci"]), "return leaf identity")
frame_fact = leaf_fact["frame"]
need(frame_fact == {"kind": "Frame::default()", "boundary": None, "scope": None}, "return frame facts shape")
need("root_frame=Frame { boundary: None" in STDERR and "scope: None" in STDERR, "incoming default frame raw")
return_record = re.search(r"CF07_RETURN_LEAVES nodes=\{(.*?)\} normal_exit=Some\((\d+)\)", STDERR)
need(return_record is not None and int(return_record.group(2)) == leaf_fact["normal_exit_node"] and int(return_record.group(1)) == leaf_fact["incoming_node"], "return leaf and normal-exit node IDs")
leaf_insn_start = STDERR.index("CF07_RETURN_LEAF node=")
leaf_insn_end = STDERR.index("CF07_RETURN_LEAF_EDGE", leaf_insn_start)
leaf_insns = [(int(bci), int(opcode)) for bci, opcode in re.findall(r"bci: (\d+),\s*opcode: (\d+),", STDERR[leaf_insn_start:leaf_insn_end])]
need((leaf_fact["terminal_bci"], leaf_fact["terminal_opcode"]) in leaf_insns, "return leaf terminal instruction")
return_edges = re.findall(r"CF07_RETURN_LEAF_EDGE CanonicalEdge \{(.*?)\n\}", STDERR, re.S)
need(len(return_edges) == 1, "return leaf has exactly one recorded edge")
def edge_pair(section):
    match = re.search(r"from: CanonicalBlockId \{\s*bci: (\d+),.*?to: CanonicalBlockId \{\s*bci: (\d+),", section, re.S)
    need(match is not None, "canonical edge endpoints")
    kind = re.search(r"kind: (Normal|Return|Exception|Call)", section)
    need(kind is not None, "canonical edge kind")
    return (kind.group(1), int(match.group(1)), int(match.group(2)))
need([f"{kind} {a} -> {b}" for kind, a, b in [edge_pair(return_edges[0])]] == leaf_fact["edges"], "return leaf incoming edge")
need(leaf_fact["outgoing_edges"] == [], "return leaf's raw edge row has no outgoing endpoint")
update_edges = re.findall(r"CF07_UPDATE_EDGE CanonicalEdge \{(.*?)\n\}", STDERR, re.S)
need([f"{kind} {a} -> {b}" for kind, a, b in map(edge_pair, update_edges)] == FACTS["update_goto_edges"], "update/goto edge facts")
update_marker = re.search(r"CF07_UPDATE_LATCH block=.*?instructions=\[(.*?)\n\]", STDERR, re.S)
need(update_marker is not None, "update/latch instruction list")
update_insns = [(int(bci), int(opcode)) for bci, opcode in re.findall(r"bci: (\d+),\s*opcode: (\d+),", update_marker.group(1))]
need((update["bci"], update["opcode"]) in update_insns and (update["goto_bci"], update["goto_opcode"]) in update_insns, "update and goto BCI/opcode facts")
need(loop_fact["latch_bcis"] == [update["bci"]], "latch BCI equals update")

expected_markers = {"CF07_INIT_INSN": 4, "CF07_INIT_VALUE": 8, "CF07_RELEVANT_PHI": 5, "CF07_SLOT4_ACCESS": 5, "CF07_NATURAL_LOOP": 1, "CF07_RETURN_LEAVES": 1, "CF07_UPDATE_LATCH": 1, "CF07_UPDATE_EDGE": 2}
actual_markers = {name: len(re.findall(r"^" + re.escape(name) + r"\b", STDERR, re.M)) for name in expected_markers}
need(actual_markers == expected_markers == RECORD["diagnostic_marker_counts"] == FACTS["cleanup"]["marker_counts"], "diagnostic marker counts")
need(FACTS["cleanup"]["restored"] == RECORD["restored"] and FACTS["cleanup"]["target_absent"] == RECORD["target_absent"], "cleanup facts")
need(f"CF07_REAL_IR_DIAG class_bytes={FACTS['class']['bytes']} class_blake3={FACTS['class']['blake3']}" in STDERR, "frozen class identity")

clean = RECORD["clean_command"]
need(clean["argv"] == ["cargo", "clean"] and clean["exit_code"] == 0 and clean["guard_stop"] is None, "clean command record")
need(FACTS["cleanup"]["clean_argv"] == clean["argv"] and FACTS["cleanup"]["clean_exit_code"] == clean["exit_code"], "clean facts bound to command record")
for stream in ("stdout", "stderr"):
    meta = clean["streams"][stream]
    raw = Path(meta["path"]).read_bytes()
    need(len(raw) == meta["bytes"] and hashlib.sha256(raw).hexdigest() == meta["sha256"], f"clean {stream} raw")
print("PASS: captured record, command/raw pins, and all key SSA facts are consistent")
