#!/usr/bin/env python3
"""Read-only verifier for root-v2 same-IR task-1.1 diagnostic evidence."""
import hashlib
import json
import re
import runpy
from pathlib import Path

ROOT = Path('/private/tmp/jarde-computed-for-builder-diagnostic-root-v2')
V9 = Path('/private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3-facts-v9')
REPO = Path('/Users/lordcasser/workspace/projects/jarde')
V1ERR = Path('/private/tmp/jarde-computed-for-builder-diagnostic-root-v1/0.stderr.raw')
RECORD = json.loads((ROOT / 'execution.json').read_text())
FACTS = json.loads((V9 / 'facts.json').read_text())
STDOUT = (ROOT / '0.stdout.raw').read_text()
STDERR_BYTES = (ROOT / '0.stderr.raw').read_bytes()
STDERR = STDERR_BYTES.decode('utf-8', 'replace')


def need(ok, msg):
    if not ok:
        raise SystemExit('FAIL: ' + msg)


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha_bytes(Path(path).read_bytes())


# The existing v9 typed SSA verifier is read and run as a separate frozen-evidence check.
# Reuse its strict typed-use parser for the v2 raw only after its own pinned root-v3 check passes.
v9ns = runpy.run_path(str(V9 / 'verify_diagnostic.py'))
v9ns['STDERR'] = STDERR
v9ns['FACTS'] = FACTS
value_section = v9ns['value_section']
def_from = v9ns['def_from']
uses_from = v9ns['uses_from']
fact_uses_match = v9ns['fact_uses_match']

need(RECORD['status'] == 'passed_same_IR_SSA_and_Builder_diagnostic_only', 'execution status is diagnostic-only pass')
need(RECORD['restored'] is True and RECORD['target_absent'] is True, 'record says byte restore and no target directory')
need(RECORD['base_region_sha256'] == 'acdb5d22195ce0b5d283fe1b7a3003f35830c85811b18e2e713b7f4728e93168', 'requested region base pin')
need(RECORD['base_build_sha256'] == 'f4660755cf2ebc699dccf8ee7f7c984a94f6695a22a62cf649d4b728c76a1ade', 'build base pin')
need(RECORD['candidate_region_sha256'] == 'f447538b3289e0de3ac8d601474c1a6442318f378c8367247a486377ffa99e81', 'tested combined region candidate pin')
need(RECORD['candidate_build_sha256'] == '02ca60d51e84c5869fe30535629dde842148e6e089e334b2197a4091590fb95b', 'tested Builder probe pin')
need(len(RECORD['source_pins_before']) == 132, 'pre-run pin count')
need(RECORD['source_pins_before'] == RECORD['source_pins_after_restore'], 'all 132 source pins restored')
changed = sorted(k for k, v in RECORD['diagnostic_source_pins'].items() if RECORD['source_pins_before'].get(k) != v)
need(changed == ['crates/jarde-java/src/build.rs', 'crates/jarde-java/src/region.rs'], 'only expected temporary test sources changed')
need(RECORD['diagnostic_source_pins']['crates/jarde-java/src/region.rs'] == RECORD['candidate_region_sha256'], 'region candidate pin binds tested source')
need(RECORD['diagnostic_source_pins']['crates/jarde-java/src/build.rs'] == RECORD['candidate_build_sha256'], 'build candidate pin binds tested source')
need(len(RECORD['source_pins_after_restore']) == 132, 'post-restore pin count')
for rel, expected in RECORD['source_pins_after_restore'].items():
    path = REPO / rel
    need(path.is_file() and sha_file(path) == expected, f'current restored file pin {rel}')

need(len(RECORD['commands']) == 3, 'command log length')
need(RECORD['commands'][0]['argv'] == ['git', 'apply', '--check', '/private/tmp/jarde-computed-for-builder-diagnostic-luna-v3/combined.patch'] and RECORD['commands'][0]['exit_code'] == 0, 'logged patch check')
need(RECORD['commands'][1]['argv'] == ['git', 'apply', '/private/tmp/jarde-computed-for-builder-diagnostic-luna-v3/combined.patch'] and RECORD['commands'][1]['exit_code'] == 0, 'logged patch application')
for i in (0, 1):
    for stream in ('stdout', 'stderr'):
        raw_path = ROOT / f'apply-{i}.{stream}.raw'
        need(sha_file(raw_path) == RECORD['commands'][i][stream + '_sha256'], f'patch command {i} {stream} raw pin')
cmd = RECORD['commands'][2]
expected_argv = ['cargo', 'test', '-p', 'jarde-java', '--lib', '--locked', 'region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir', '--', '--exact', '--nocapture']
need(cmd['argv'] == expected_argv and cmd['exit_code'] == 0 and cmd['guard_stop'] is None, 'exact test argv and successful completion')
need(cmd['cwd'] == str(REPO), 'test cwd')
for stream, path in (('stdout', ROOT / '0.stdout.raw'), ('stderr', ROOT / '0.stderr.raw')):
    meta = cmd['streams'][stream]
    raw = path.read_bytes()
    need(len(raw) == meta['bytes'] and sha_bytes(raw) == meta['sha256'], f'{stream} byte count and raw SHA')
need('running 1 test' in STDOUT and '1 passed; 0 failed; 0 ignored; 0 measured; 350 filtered out' in STDOUT, 'single exact test summary')
clean = RECORD['clean_command']
need(clean['argv'] == ['cargo', 'clean'] and clean['exit_code'] == 0 and clean['guard_stop'] is None, 'post-run cleanup command')
for stream in ('stdout', 'stderr'):
    meta = clean['streams'][stream]
    need(sha_file(ROOT / f'1.{stream}.raw') == meta['sha256'], f'clean {stream} raw SHA')
need(RECORD['target_absent'] is True and RECORD['guards']['free_bytes'] == 5 * 1024**3 and RECORD['guards']['target_bytes'] == 1024**3, 'guard and target cleanup state')

# Pin frozen test class and match the v9 typed fact ledger to this combined test's raw.
class_rel = 'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes/cf07/LoopCases.class'
need(RECORD['source_pins_before'][class_rel] == FACTS['class']['class_sha256'], 'frozen class SHA-256 matches v9 facts')
need(f'CF07_REAL_IR_DIAG class_bytes={FACTS["class"]["bytes"]} class_blake3={FACTS["class"]["blake3"]}' in STDERR, 'same class bytes and BLAKE3 in v2 raw')
need('method=lastIndexOf([IIII)I header=CanonicalBlockId { bci: 5' in STDERR, 'real method/header identity in raw')
need('root_frame=Frame { boundary: None' in STDERR and 'scope: None' in STDERR, 'real incoming default frame')

# Exact initialization instructions/slot accesses and strict typed def/use checks from v9 facts.
raw_insns = [(int(b), int(o)) for b, o in re.findall(r'CF07_INIT_INSN .*?instruction=SsaInstruction \{\n\s*bci: (\d+),\n\s*opcode: (\d+),', STDERR, re.S)]
init = FACTS['ssa_init']
need(raw_insns == [(row['bci'], row['opcode']) for row in init], 'four initialization instructions and opcodes bound to raw')
expected_init_accesses = []
for row in init:
    expected_init_accesses += [(row['bci'], entry['slot'], entry['value']) for entry in row['reads']]
    expected_init_accesses += [(row['bci'], entry['slot'], entry['value']) for entry in row['writes']]
raw_init_accesses = [(int(b), slot, f'ValueId({v})') for b, slot, v in re.findall(r'^CF07_INIT_VALUE bci=(\d+) access_slot=([^ ]+) value_id=ValueId\((\d+)\)', STDERR, re.M)]
need(raw_init_accesses == expected_init_accesses, 'all initialization SSA reads/writes bound to v9 facts')
for row, value_name in zip(init, ('value9', 'value10', 'value11', 'value12')):
    write = row['writes'][-1]
    value_id = int(re.search(r'ValueId\((\d+)\)', write['value']).group(1))
    text = value_section(value_id, row['bci'], write['slot'])
    need(def_from(text) == row[value_name]['def'], f'{value_name} exact instruction definition')
    fact_uses_match(uses_from(text), row[value_name]['uses'], value_id)
source_text = value_section(3, 0, 'Local(3)')
need(def_from(source_text) == init[0]['source_value3']['def'], 'arg3 is entry local 3')
fact_uses_match(uses_from(source_text), init[0]['source_value3']['uses'], 3)

# Header phi, full slot-4 access set, natural loop, terminal leaf/frame and exact update/goto edges.
phi = FACTS['header_phi']
phi_start = STDERR.rfind('CF07_RELEVANT_PHI SsaPhi {')
phi_end = STDERR.index('CF07_SLOT4_ACCESS', phi_start)
phi_text = STDERR[phi_start:phi_end]
need(re.search(r'block: CanonicalBlockId \{\s*bci: 5,', phi_text) is not None and re.search(r'slot: Local\(\s*4,\s*\)', phi_text) is not None and re.search(r'value: ValueId\(\s*8,\s*\)', phi_text) is not None, 'header local-4 phi identity')
raw_phi_inputs = [f'ValueId({n})' for n in re.findall(r'ValueId\(\s*(\d+),\s*\)', phi_text[phi_text.find('inputs:'):phi_text.find('value=SsaValue')])]
need(raw_phi_inputs == [row['value'] for row in phi['inputs']], 'both header phi inputs and order')
phi_value = value_section(8, 5, 'Local(4)', 'CF07_SLOT4_VALUE')
need(def_from(phi_value) == phi['definition'], 'header phi definition')
fact_uses_match(uses_from(phi_value), phi['uses'], 8)
expected_slot4 = []
for row in FACTS['local4_accesses']:
    if 'read' in row: expected_slot4.append((row['bci'], row['read']))
    if 'write' in row: expected_slot4.append((row['bci'], row['write']))
raw_slot4 = [(int(b), f'ValueId({v})') for b, v in re.findall(r'^CF07_SLOT4_VALUE bci=(\d+) slot=Local\(4\) value_id=ValueId\((\d+)\)', STDERR, re.M)]
need(raw_slot4 == expected_slot4, 'complete ordered local-4 access set')
update = FACTS['update_value']
update_value = value_section(21, 22, 'Local(4)', 'CF07_SLOT4_VALUE')
need(def_from(update_value) == update['value21_def'], 'update value definition')
fact_uses_match(uses_from(update_value), [update['value21_use']], 21)
loop = FACTS['natural_loop']
loop_text = STDERR[STDERR.index('CF07_NATURAL_LOOP NaturalLoop {'):STDERR.index('CF07_LOOP_BLOCK')]
need(re.search(r'header: (\d+),', loop_text).group(1) == str(loop['header_node']), 'natural-loop header node')
blocks = re.search(r'blocks: \{([^}]*)\}', loop_text, re.S)
latches = re.search(r'latches: \{([^}]*)\}', loop_text, re.S)
need([int(n) for n in re.findall(r'\d+', blocks.group(1))] == loop['block_nodes'], 'natural-loop block node set')
need([int(n) for n in re.findall(r'\d+', latches.group(1))] == loop['latch_nodes'], 'natural-loop latch nodes')
loop_bcis = [int(n) for n in re.findall(r'^CF07_LOOP_BLOCK node=\d+ id=CanonicalBlockId \{ bci: (\d+),', STDERR, re.M)]
need(loop_bcis == loop['block_bcis'] and loop['latch_bcis'] == [update['bci']], 'loop-node BCI mapping and latch')
need('irreducible: false' in loop_text, 'reducible natural loop')
leaf = FACTS['return_leaf']
need(re.search(r'CF07_RETURN_LEAVES nodes=\{(\d+)\} normal_exit=Some\((\d+)\)', STDERR).groups() == (str(leaf['incoming_node']), str(leaf['normal_exit_node'])), 'terminal-return certificate leaf and normal exit')
need(re.search(r'CF07_RETURN_LEAF node=(\d+) id=CanonicalBlockId \{ bci: (\d+),', STDERR).groups() == (str(leaf['incoming_node']), str(leaf['block_bci'])), 'return leaf physical block')
leaf_segment = STDERR[STDERR.index('CF07_RETURN_LEAF node='):STDERR.index('CF07_RETURN_LEAF_EDGE')]
need((leaf['terminal_bci'], leaf['terminal_opcode']) in [(int(b), int(o)) for b, o in re.findall(r'bci: (\d+),\s*opcode: (\d+),', leaf_segment)], 'return terminal BCI/opcode')
need(leaf['frame'] == {'kind': 'Frame::default()', 'boundary': None, 'scope': None}, 'frame facts ledger')
need([x for x in FACTS['update_goto_edges']] == ['Normal 11 -> 22', 'Normal 22 -> 5'], 'update/goto edge expected facts')
raw_update_edges = re.findall(r'CF07_UPDATE_EDGE CanonicalEdge \{(.*?)\n\}', STDERR, re.S)
for expected in FACTS['update_goto_edges']:
    kind, edge = expected.split(' ', 1)
    src, dst = map(int, re.search(r'(\d+) -> (\d+)', edge).groups())
    need(any(f'kind: {kind}' in text and re.search(rf'from: CanonicalBlockId \{{\s*bci: {src},.*?to: CanonicalBlockId \{{\s*bci: {dst},', text, re.S) for text in raw_update_edges), f'canonical edge {expected}')

# Builder behavior: preserve the actual current presentation, do not claim a For was implemented.
bm = re.search(r'CF07_BUILDER_MATERIALIZATION stmts=(.*?)\nCF07_BUILDER_PROGRAM', STDERR, re.S)
need(bm is not None, 'Builder materialization record')
for empty_state in ('deferred=[]', 'plans={}', 'bindings={}', 'rejected={}', 'binding_refused={}'): need(empty_state in bm.group(1), f'empty Builder state {empty_state}')
need('CF07_BUILDER_PROGRAM init_store_bci=Some(3) statements=7 ragged=false' in STDERR, 'whole Builder output count and ragged state')
stmt_block = STDERR[STDERR.index('CF07_BUILDER_INIT_STMT'):STDERR.index('CF07_BUILDER_ALL_STMTS')]
need(len(re.findall(r'^CF07_BUILDER_INIT_STMT Stmt \{', stmt_block, re.M)) == 2, 'exactly two init statements at the Store origin')
need('kind: Declare {' in stmt_block and 'value: None' in stmt_block, 'first init statement is empty local declaration')
need('kind: Assign {' in stmt_block and 'op: Subtract' in stmt_block, 'second init statement assigns subtraction')
need(re.search(r'kind: Local\(\s*"arg3",', stmt_block) is not None and re.search(r'kind: Integer\(\s*1,', stmt_block) is not None, 'assignment operands are arg3 and literal 1')
need('bci: 0,' in stmt_block and 'bci: 1,' in stmt_block and 'bci: 2,' in stmt_block and stmt_block.count('bci: 3,') == 2, 'expression and both statement anchors are the actual bytecodes')
need('kind: For {' not in STDERR and 'kind: While {' in STDERR, 'current output is while-shaped; no For recovery claim')
frame_rows = re.findall(r'^CF07_BUILDER_FRAME bci=(\d+) operation=(.*?) instruction=', STDERR, re.M)
need(len(frame_rows) == 4 and [int(row[0]) for row in frame_rows] == [0, 1, 2, 3], 'Builder frame context covers exactly real instructions 0..3')
need(len(re.findall(r'^CF07_BUILDER_FRAME_VALUE ', STDERR, re.M)) == 8, 'all eight Builder frame access values recorded')
need('CF07_BUILDER_FRAME bci=3 operation=Some(Store { slot: 4 })' in STDERR, 'actual preheader Store instruction')
need('CF07_BUILDER_FRAME_VALUE bci=3 slot=Stack(0) id=ValueId(11) ty=Int def=Instruction { block: CanonicalBlockId { bci: 0, path: [] }, bci: 2 } uses=[SsaUse { block: CanonicalBlockId { bci: 0, path: [] }, bci: Some(3) }]' in STDERR, 'store input is computed at BCI 2 and consumed by store at BCI 3')
need('CF07_BUILDER_FRAME_VALUE bci=3 slot=Local(4) id=ValueId(12) ty=Int def=Instruction { block: CanonicalBlockId { bci: 0, path: [] }, bci: 3 } uses=[SsaUse { block: CanonicalBlockId { bci: 5, path: [] }, bci: None }]' in STDERR, 'Store result is distinct SSA local value feeding header phi')

# Confirm the preserved root-v1 failure documents the two fixed issues and remains separate.
v1err = V1ERR.read_text(errors='replace')
need("cannot find `ArrayInitializers` in `init`" in v1err and 'expected `u16`, found `Slot`' in v1err, 'v1 compile errors retained as historical evidence')
print('PASS: root-v2 raw, run/source pins, v9 typed SSA facts, Builder statement/materialization, restore and cleanup verified')
print('TEST_ARGV=' + json.dumps(cmd['argv'], separators=(',', ':')))
print('SCOPE=task 1.1 diagnostic evidence only; computed-for 2.x implementation not established')
