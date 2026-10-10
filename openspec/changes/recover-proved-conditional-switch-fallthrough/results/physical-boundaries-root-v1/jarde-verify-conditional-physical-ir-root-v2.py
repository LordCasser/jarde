"""Independent acceptance of the frozen conditional physical-IR observation."""

from collections import Counter, defaultdict, deque
import ast
import hashlib
import json
from pathlib import Path
import re

import blake3


REPO = Path("/Users/lordcasser/workspace/projects/jarde")
ROOT = Path("/private/tmp/jarde-conditional-physical-ir-root-v1")
CLASS_ROOT = Path("/private/tmp/jarde-conditional-physical-root-v1/cases")
EXECUTION = ROOT / "execution.json"
STDOUT = ROOT / "0.stdout.raw"
STDERR = ROOT / "0.stderr.raw"
OBSERVER = ROOT / "observer-root.rs"
TEMP_TEST = REPO / "crates/jarde-java/tests/__root_conditional_physical_ir_observer.rs"
TARGET = REPO / "target"
PIN_MANIFEST = REPO / (
    "openspec/changes/recover-proved-local-source-types/results/"
    "candidate-cli-typed-root-v1.json"
)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise AssertionError(message)


execution = json.loads(EXECUTION.read_text())
require(
    execution["schema"] == "conditional-physical-public-ir-observation-root-v1",
    "unexpected execution schema",
)
require(execution["status"] == "public-ir-observations-collected", "observation status changed")
require(execution["product_accepted"] is False, "observation must not imply product acceptance")
require(execution["temporary_test_removed"] is True, "temporary observer test was not recorded removed")
require(execution["frozen_sources_unchanged"] is True, "frozen source stability was not recorded")

require(sha256(PIN_MANIFEST) == "6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943", "frozen manifest SHA mismatch")
manifest = json.loads(PIN_MANIFEST.read_text())
live_pins = {
    path: digest
    for group in ("candidate_sources", "test_sources", "canonical_files")
    for path, digest in manifest[group].items()
}
repair_path = Path("/private/tmp/jarde-typed-ci-repair-root-v7/execution.json")
require(sha256(repair_path) == "860f4817e9e82566aeb37b791a99de8d07311ac826fd1385938b63e0002e9559", "focused repair record SHA mismatch")
live_pins.update(json.loads(repair_path.read_text())["additional_test_pins"])
live_pins["tests/p5_bulk_corpus.rs"] = "48ec01a4123021db3b4f6e5942adfbc1bba343009741a7edd767dd15d9009fee"
require(len(live_pins) == 86, "exact source pin closure differs")
require(execution["source_pins_before"] == execution["source_pins_after"], "source pins changed during observation")
require(execution["source_pins_before"] == live_pins, "execution pins differ from frozen source manifest")
for relative, expected in live_pins.items():
    require(sha256(REPO / relative) == expected, f"live source pin changed: {relative}")

require(not TEMP_TEST.exists(), "temporary observer test still exists")
require(not TARGET.exists(), "repository target directory exists")
require(sha256(OBSERVER) == execution["observer_source_sha256"], "observer source hash mismatch")

require(len(execution["commands"]) == 1, "expected exactly one guarded observation command")
command = execution["commands"][0]
require(command["index"] == 0, "unexpected command index")
require(command["argv"] == [
    "cargo", "test", "-p", "jarde-java", "--test",
    "__root_conditional_physical_ir_observer", "--locked", "--", "--nocapture",
], "guarded command argv changed")
require(command["cwd"] == str(REPO), "guarded command cwd changed")
require(command["exit_code"] == 0 and command["guard_stop"] is None, "guarded observation failed or stopped")
require(command["peak_target_bytes"] == 180560145, "observed peak target bytes changed")
require(command["free_bytes_after"] >= 5 * 1024**3, "recorded free space is below guard threshold")

for stream_name, path in (("stdout", STDOUT), ("stderr", STDERR)):
    stream = command["streams"][stream_name]
    require(Path(stream["path"]) == path, f"{stream_name} path mismatch")
    require(path.parent == ROOT, f"{stream_name} escaped observation root")
    require(path.stat().st_size == stream["bytes"], f"{stream_name} size mismatch")
    require(sha256(path) == stream["sha256"], f"{stream_name} raw SHA-256 mismatch")

raw = STDOUT.read_text()
summaries = re.findall(
    r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in ([^\n]+)",
    raw,
)
require(len(summaries) == 1 and summaries[0][:5] == ("1", "0", "0", "0", "0"), "raw test summary is not 1/0/0")

expected_legs = {
    "javac8-A-multiple-exit": ("javac8", "A-multiple-exit"),
    "javac8-B-nonadjacent": ("javac8", "B-nonadjacent"),
    "javac23-A-multiple-exit": ("javac23", "A-multiple-exit"),
    "javac23-B-nonadjacent": ("javac23", "B-nonadjacent"),
}
class_inputs = {}
for leg, length, sha, digest in re.findall(
    r"CLASS_INPUT leg=(\S+) length=(\d+) sha256=([0-9a-f]{64}) blake3=([0-9a-f]{64})", raw
):
    require(leg not in class_inputs, f"duplicate class input row: {leg}")
    require(leg in expected_legs, f"unexpected class input row: {leg}")
    compiler, case = expected_legs[leg]
    path = CLASS_ROOT / compiler / case / "classes/ConditionalSwitchBoundaries.class"
    data = path.read_bytes()
    require(len(data) == int(length), f"class length mismatch: {leg}")
    require(hashlib.sha256(data).hexdigest() == sha, f"class SHA-256 mismatch: {leg}")
    require(blake3.blake3(data).hexdigest() == digest, f"class BLAKE3 mismatch: {leg}")
    class_inputs[leg] = {"path": str(path), "bytes": data, "sha256": sha, "blake3": digest}
require(set(class_inputs) == set(expected_legs), "raw class input rows are incomplete")

method_rows = re.findall(
    r"^METHOD_BEGIN class_leg=(\S+) owner=(\S+) name=(\S+) descriptor=(\S+)\n(.*?)"
    r"^METHOD_END class_leg=(\S+) name=(\S+) descriptor=(\S+)$",
    raw,
    re.M | re.S,
)
require(len(method_rows) == 4, "expected four raw method blocks")
require({row[0] for row in method_rows} == set(expected_legs), "method labels differ from class inputs")


def decode_jvm_bytes(text):
    values = [int(item) for item in re.findall(r"\d+", text)]
    return bytes(values).decode("ascii")


profiles = []
for leg, owner, name, descriptor, body, end_leg, end_name, end_descriptor in method_rows:
    require(owner == "ConditionalSwitchBoundaries" and name == "partialBreak", f"wrong method identity: {leg}")
    require(descriptor == "(II)Ljava/lang/String;", f"wrong method descriptor: {leg}")
    require((end_leg, end_name, end_descriptor) == (leg, name, descriptor), f"method close mismatch: {leg}")
    source = class_inputs[leg]
    input_digest = source["blake3"]
    report = body.split("PHYSICAL_CODE ", 1)[0]
    require(input_digest in report and "execution: Complete {" in report, f"report does not identify complete input: {leg}")

    stage_rows = re.findall(r"StageResult \{\s*stage: (\w+),\s*state: (\w+),", report)
    expected_stages = ["RawFacts", "RawCfg", "LegacyNormalization", "CanonicalCfg", "Frame", "Ssa"]
    require(stage_rows == [(stage, "Completed") for stage in expected_stages], f"analysis stages incomplete: {leg}")

    snapshot_match = re.search(
        r"location: StandaloneRoot \{\s*snapshot: SnapshotId\(\s*\"([0-9a-f]{64})\",?\s*\),?\s*\}",
        report,
        re.S,
    )
    class_id_match = re.search(
        r"class_bytes: ClassBytesId \{\s*digest: Digest\(\s*\"([0-9a-f]{64})\",?\s*\),\s*length: (\d+),",
        report,
        re.S,
    )
    method_match = re.search(
        r"method: PhysicalMethodId \{.*?name: JvmBytes\(\s*\[([^]]*)\],?\s*\),"
        r"\s*descriptor: JvmBytes\(\s*\[([^]]*)\],?\s*\)",
        report,
        re.S,
    )
    require(snapshot_match and class_id_match and method_match, f"physical identity incomplete: {leg}")
    snapshot = snapshot_match.group(1)
    class_digest, class_length = class_id_match.groups()
    require(snapshot == input_digest == class_digest, f"physical owner/snapshot digest mismatch: {leg}")
    require(int(class_length) == len(source["bytes"]), f"physical owner length mismatch: {leg}")
    require(decode_jvm_bytes(method_match.group(1)) == name, f"physical method name mismatch: {leg}")
    require(decode_jvm_bytes(method_match.group(2)) == descriptor, f"physical method descriptor mismatch: {leg}")

    physical = [
        (int(bci), int(opcode, 16), int(width))
        for bci, opcode, width in re.findall(
            r"^PHYSICAL_INSTRUCTION bci=(\d+) opcode=0x([0-9a-f]{2}) width=(\d+)", body, re.M
        )
    ]
    typed = [
        (int(bci), int(opcode, 16), int(effective, 16), int(width), operands)
        for bci, opcode, effective, width, operands in re.findall(
            r"^PHYSICAL_TYPED_INSTRUCTION bci=(\d+) opcode=0x([0-9a-f]{2}) "
            r"effective_opcode=0x([0-9a-f]{2}) width=(\d+) operands=(.*)$",
            body,
            re.M,
        )
    ]
    require(physical and len(physical) == len(typed), f"physical/typed decode row count differs: {leg}")
    require(physical == [(bci, opcode, width) for bci, opcode, effective, width, _ in typed], f"physical/typed decode differs: {leg}")
    require(all(effective == opcode for _, opcode, effective, _, _ in typed), f"effective opcode mismatch: {leg}")
    require(physical[0][0] == 0 and all(a[0] + a[2] == b[0] for a, b in zip(physical, physical[1:])), f"physical decode has a gap: {leg}")

    typed_by_bci = {bci: (opcode, operands) for bci, opcode, effective, width, operands in typed}
    require(typed_by_bci[37][0] == 0x99 and typed_by_bci[47][0] == 0xA7, f"branch opcodes differ: {leg}")
    offset37 = re.search(r"branch_offset: Some\((-?\d+)\)", typed_by_bci[37][1])
    offset47 = re.search(r"branch_offset: Some\((-?\d+)\)", typed_by_bci[47][1])
    require(offset37 and offset47, f"branch operands missing: {leg}")
    target37 = 37 + int(offset37.group(1))
    target47 = 47 + int(offset47.group(1))
    flavor = expected_legs[leg][1]
    expected37 = 57 if flavor == "A-multiple-exit" else 67
    require(target37 == expected37 and target47 == 67, f"ifeq/goto destinations differ: {leg}")

    targets = [
        (int(instruction), kind, int(target))
        for instruction, kind, target in re.findall(
            r"^PHYSICAL_TARGET ControlFlowTarget \{ instruction_bci: (\d+), kind: (.*), target_bci: (\d+) \}$",
            body,
            re.M,
        )
    ]
    switch_targets = [(kind, target) for instruction, kind, target in targets if instruction == 9]
    require(Counter(switch_targets) == Counter([
        ("SwitchDefault", 67),
        ("SwitchCase { index: 0, key: 0 }", 36),
        ("SwitchCase { index: 1, key: 1 }", 57),
    ]), f"switch destinations differ: {leg}")
    require((37, f"Branch {{ offset: {target37 - 37} }}", target37) in targets, f"ifeq target absent from physical targets: {leg}")
    require((47, "Branch { offset: 20 }", 67) in targets, f"goto target absent from physical targets: {leg}")

    require("CANONICAL completeness=Complete unreachable=[]" in body, f"canonical completeness differs: {leg}")
    blocks = [
        (int(bci), ast.literal_eval(path), int(end))
        for bci, path, end in re.findall(
            r"^CANONICAL_BLOCK bci=(\d+) path=(\[[^]]*\]) end_bci=(\d+)", body, re.M
        )
    ]
    require(blocks and all(path == [] for _, path, _ in blocks), f"canonical paths are not empty: {leg}")
    require(len({bci for bci, _, _ in blocks}) == len(blocks), f"duplicate canonical block: {leg}")
    block_map = {bci: end for bci, _, end in blocks}
    require(block_map == {0: 36, 36: 40, 40: 50, 57: 67, 67: 74, 74: 79}, f"canonical block boundaries differ: {leg}")
    instruction_starts = {bci for bci, _, _ in physical}
    require({36, 57, 67} <= instruction_starts, f"switch target is not a physical instruction entry: {leg}")
    require(physical[-1][0] + physical[-1][2] == 79, f"physical decode extent differs: {leg}")
    omitted = []
    for bci, _, width in physical:
        coverage = sum(start <= bci < end for start, _, end in blocks)
        require(coverage <= 1, f"overlapping canonical intervals: {leg}:{bci}")
        if coverage == 0: omitted.append(bci)
    require(omitted == [50, 51, 53, 56], f"observed canonical omission differs: {leg}")

    edges = [
        (int(source), tuple(ast.literal_eval(source_path)), kind, int(target), tuple(ast.literal_eval(target_path)))
        for source, source_path, kind, target, target_path in re.findall(
            r"^CANONICAL_EDGE from=(\d+) from_path=(\[[^]]*\]) kind=(.*?) to=(\d+) to_path=(\[[^]]*\])$",
            body,
            re.M,
        )
    ]
    block_ids = set(block_map)
    require(len(edges) == 8 and all(kind == "Normal" for _, _, kind, _, _ in edges), f"expected eight normal canonical edges: {leg}")
    require(all(src in block_ids and dst in block_ids and src_path == () and dst_path == () for src, src_path, _, dst, dst_path in edges), f"edge endpoint/path outside raw canonical blocks: {leg}")
    edge_multiset = Counter((src, src_path, kind, dst, dst_path) for src, src_path, kind, dst, dst_path in edges)
    common_edges = [
        (0, (), "Normal", 36, ()),
        (0, (), "Normal", 57, ()),
        (0, (), "Normal", 67, ()),
        (36, (), "Normal", 40, ()),
        (40, (), "Normal", 67, ()),
        (57, (), "Normal", 74, ()),
        (67, (), "Normal", 74, ()),
    ]
    expected_edges = Counter(common_edges + [
        (36, (), "Normal", 57 if flavor == "A-multiple-exit" else 67, ())
    ])
    require(edge_multiset == expected_edges, f"full canonical edge multiset differs: {leg}")

    ssa_blocks = [
        (int(bci), tuple(ast.literal_eval(path)))
        for bci, path in re.findall(r"^SSA_BLOCK bci=(\d+) path=(\[[^]]*\])", body, re.M)
    ]
    require(Counter(ssa_blocks) == Counter((bci, ()) for bci in block_ids), f"SSA membership differs from canonical blocks: {leg}")

    incoming = defaultdict(list)
    adjacency = defaultdict(list)
    for src, _, kind, dst, _ in edges:
        incoming[dst].append((src, kind))
        adjacency[src].append(dst)
    # Read every incoming row from the complete edge multiset before deriving exits.
    incoming_boundary = {
        str(target): sorted(incoming[target]) for target in (36, 57, 67)
    }
    exits = set()
    pending = deque([36])
    seen_nodes = set()
    while pending:
        node = pending.popleft()
        if node in seen_nodes:
            continue
        seen_nodes.add(node)
        if node in (57, 67) and node != 36:
            exits.add(node)
            continue
        pending.extend(adjacency[node])
    expected_exits = {57, 67} if flavor == "A-multiple-exit" else {67}
    require(exits == expected_exits, f"case 0 reachable exits differ: {leg}")

    profiles.append({
        "leg": leg,
        "owner": owner,
        "method": name,
        "descriptor": descriptor,
        "class_path": source["path"],
        "class_sha256": source["sha256"],
        "class_blake3": input_digest,
        "physical_owner_snapshot_blake3": snapshot,
        "physical_instruction_count": len(physical),
        "physical_instruction_bcis_not_listed_in_canonical": omitted,
        "canonical_blocks": [{"bci": bci, "path": path, "end_bci": end} for bci, path, end in blocks],
        "canonical_edges_full_multiset": [
            {"from": src, "from_path": list(src_path), "kind": kind, "to": dst, "to_path": list(dst_path)}
            for src, src_path, kind, dst, dst_path in edges
        ],
        "incoming_to_switch_targets": incoming_boundary,
        "physical_switch_targets": {"case0": 36, "case1": 57, "default": 67},
        "ifeq_37_target": target37,
        "goto_47_target": target47,
        "case0_reachable_exits": sorted(exits),
        "ssa_block_membership": sorted(bci for bci, _ in ssa_blocks),
    })

require(not TARGET.exists(), "repository target directory appeared during verification")
result = {
    "schema": "conditional-physical-public-ir-observations-acceptance-root-v1",
    "status": "accepted-public-ir-observations-no-product",
    "execution_sha256": sha256(EXECUTION),
    "stdout_sha256": sha256(STDOUT),
    "stderr_sha256": sha256(STDERR),
    "observer_sha256": execution["observer_source_sha256"],
    "guarded_command_peak_target_bytes": command["peak_target_bytes"],
    "profiles": profiles,
    "product_acceptance": False,
    "scope": "public physical decode, canonical CFG, SSA block membership, on four frozen class inputs; no conditional product proof or acceptance",
}
print(json.dumps(result, indent=2, sort_keys=True))
