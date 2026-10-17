#!/usr/bin/env python3
"""Read-only acceptance for recover-return-through-nested-finally-cleanup.

Consumes compare.py's completed execution.json, its source JSON and raw streams,
the sealed EX.class, and the pinned baseline-v2 capture. It does not invoke tools.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile

BASELINE_SHA256 = 'd154b9c99a326da6f30843502d5edeb77a9be81cede65fa14ea46b773ee07e7a'
SEALED_EX_SHA256 = 'e6f0eb1e029a2c883d3ee5df5e089eda17fea27833a9244297cede139cd61287'
BASELINE_MEMBERS = {
    'default': 'matrix/sources/sealed-ex/default.json',
    'all': 'matrix/sources/sealed-ex/all.json',
}
METHODS = [
    ('<init>', '()V'),
    ('multi', '(Ljava/lang/String;)Ljava/lang/String;'),
    ('finReturn', '(I)I'),
    ('finThrow', '(I)Ljava/lang/String;'),
    ('nestedFin', '(I)I'),
    ('main', '([Ljava/lang/String;)V'),
]
RUNTIMES = ('javac8', 'javac23')
INPUTS = ('javac8', 'javac23')
EVIDENCE = ('default', 'all')


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for part in iter(lambda: f.read(1024 * 1024), b''):
            h.update(part)
    return h.hexdigest()


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def read_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))


def u8(data: bytes, p: int) -> tuple[int, int]:
    need(p + 1 <= len(data), 'truncated classfile u1')
    return data[p], p + 1


def u2(data: bytes, p: int) -> tuple[int, int]:
    need(p + 2 <= len(data), 'truncated classfile u2')
    return int.from_bytes(data[p:p + 2], 'big'), p + 2


def u4(data: bytes, p: int) -> tuple[int, int]:
    need(p + 4 <= len(data), 'truncated classfile u4')
    return int.from_bytes(data[p:p + 4], 'big'), p + 4


def classfile_method_order(path: Path) -> list[tuple[str, str]]:
    data = path.read_bytes()
    need(data[:4] == b'\xca\xfe\xba\xbe', 'EX input is not a classfile')
    p = 8
    cp_count, p = u2(data, p)
    cp = [None] * cp_count
    i = 1
    while i < cp_count:
        tag, p = u8(data, p)
        if tag == 1:
            n, p = u2(data, p)
            cp[i] = data[p:p + n].decode('utf-8', errors='replace')
            p += n
        elif tag in (3, 4): p += 4
        elif tag in (5, 6): p += 8; i += 1
        elif tag in (7, 8, 16, 19, 20): _, p = u2(data, p)
        elif tag in (9, 10, 11, 12, 17, 18): p += 4
        elif tag == 15: p += 3
        else: raise ValueError(f'unsupported constant pool tag {tag}')
        need(p <= len(data), 'truncated constant pool')
        i += 1

    def attrs(pos: int) -> int:
        count, pos = u2(data, pos)
        for _ in range(count):
            _, pos = u2(data, pos)
            size, pos = u4(data, pos)
            pos += size
            need(pos <= len(data), 'truncated classfile attribute')
        return pos

    p += 6
    interfaces, p = u2(data, p)
    p += 2 * interfaces
    fields, p = u2(data, p)
    for _ in range(fields):
        p += 6
        p = attrs(p)
    methods, p = u2(data, p)
    result = []
    for _ in range(methods):
        p += 2
        name_i, p = u2(data, p)
        desc_i, p = u2(data, p)
        p = attrs(p)
        result.append((cp[name_i], cp[desc_i]))
    return result


def method_key(method: dict) -> tuple[str, str]:
    identity = method['item']['identity']
    return (bytes(identity['name']).decode('utf-8'),
            bytes(identity['descriptor']).decode('utf-8'))


def command_stream(commands: dict[int, dict], command_id: int, channel: str, root: Path) -> bytes:
    row = commands[command_id]
    record = row['streams'][channel]
    path = Path(record['path'])
    if not path.is_absolute():
        path = root / path
    content = path.read_bytes()
    need(len(content) == record['bytes'], f'command {command_id} {channel}: byte length changed')
    need(hashlib.sha256(content).hexdigest() == record['sha256'],
         f'command {command_id} {channel}: raw stream digest changed')
    return content


def check_18_case_stdout(data: bytes, label: str) -> None:
    lines = data.decode('utf-8', errors='strict').splitlines()
    need(len(lines) == 19 and lines[-1] == 'nested-finally: ok',
         f'{label}: expected 18 cases and completion marker')
    for n in (2, 0):
        for in_kind in range(3):
            for out_kind in range(3):
                prefix = f'n={n},in={in_kind},out={out_kind}:'
                found = [line for line in lines[:-1] if line.startswith(prefix)]
                need(len(found) == 1, f'{label}: missing/duplicate {prefix}')
                line = found[0]
                need(line.endswith(':events=IO'), f'{label}: cleanup order/count differs for {prefix}')
                kind = out_kind if out_kind else in_kind
                if kind:
                    typ, msg = (('java.lang.RuntimeException', 'out-runtime' if out_kind else 'in-runtime')
                                if kind == 1 else
                                ('java.lang.AssertionError', 'out-error' if out_kind else 'in-error'))
                    need(line == f'{prefix} throw={typ}:message={msg}:sameInjectedObject=true:events=IO',
                         f'{label}: thrown type/message/identity differs for {prefix}')
                elif n == 0:
                    need(line.startswith(prefix + ' throw=java.lang.ArithmeticException:message='),
                         f'{label}: original division exception missing for {prefix}')
                else:
                    need(line == prefix + ' return=5:events=IO', f'{label}: return differs for {prefix}')


def source_document(out: Path, commands: dict[int, dict], command_id: int) -> tuple[dict, bytes]:
    row = commands[command_id]
    artifact = row.get('json_artifact')
    need(isinstance(artifact, dict) and isinstance(artifact.get('path'), str),
         f'class-source command {command_id} has no JSON artifact pin')
    path = Path(artifact['path'])
    raw = path.read_bytes()
    need(len(raw) == artifact['bytes'] and hashlib.sha256(raw).hexdigest() == artifact['sha256'],
         f'class-source JSON artifact pin mismatch: {path}')
    return json.loads(raw.decode('utf-8')), raw


def validate_ex_document(doc: dict, class_facts, expected_order, profile: str, *,
                         require_nested_structured: bool = True,
                         require_bci_coverage: bool = True) -> dict[tuple[str, str], dict]:
    methods = doc.get('methods')
    need(isinstance(methods, list) and len(methods) == 6, f'{profile}: EX must report all six methods')
    need([(method_key(m)) for m in methods] == expected_order,
         f'{profile}: method ordinal/order/identity differs from EX.class')
    need([m['item'].get('index') for m in methods] == list(range(6)),
         f'{profile}: method ordinals are not exactly 0..5')
    by_key = {}
    for method, (name, desc) in zip(methods, expected_order):
        key = (name, desc)
        report = method.get('outcome', {}).get('report')
        need(isinstance(report, dict), f'{profile} {name}{desc}: missing method report')
        identity = method['item']['identity']
        binding = report.get('artifact', {}).get('binding', {})
        need(report.get('method') == name + desc,
             f'{profile} {name}{desc}: report method signature differs')
        need(binding.get('method') == identity,
             f'{profile} {name}{desc}: artifact binding method identity differs')
        need(binding.get('member_ordinal') == method['item']['index'],
             f'{profile} {name}{desc}: artifact member ordinal differs')
        need(identity['owner']['class_bytes'] == doc['class']['class_bytes'],
             f'{profile} {name}{desc}: owner digest is not this EX definition')
        need(identity['owner']['class_bytes']['length'] == class_facts['length'],
             f'{profile} {name}{desc}: owner length differs from pinned class bytes')
        physical_bcis = class_facts['methods'][key]['instruction_bcis']
        smap = report.get('source_map')
        need(isinstance(smap, dict) and isinstance(smap.get('segments'), list),
             f'{profile} {name}{desc}: source map missing')
        report_text = report.get('text')
        need(isinstance(report_text, str), f'{profile} {name}{desc}: report text missing')
        text_bytes = len(report_text.encode('utf-8'))
        seen_bcis = set()
        for segment in smap['segments']:
            start, end = segment.get('start'), segment.get('end')
            need(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= text_bytes,
                 f'{profile} {name}{desc}: source-map range is outside report UTF-8 bytes')
            origin = segment.get('origin')
            need(isinstance(origin, dict) and isinstance(origin.get('primary'), dict),
                 f'{profile} {name}{desc}: malformed source-map origin')
            for item in [origin['primary'], *origin.get('derived', [])]:
                need(item.get('method') == identity,
                     f'{profile} {name}{desc}: origin method identity differs')
                bci = item.get('bci')
                need(isinstance(bci, int) and bci in physical_bcis,
                     f'{profile} {name}{desc}: origin BCI is not a physical instruction boundary: {bci}')
                seen_bcis.add(bci)
        if require_bci_coverage:
            need(seen_bcis == set(physical_bcis),
                 f'{profile} {name}{desc}: source map does not account for every physical BCI; '
                 f'missing={sorted(set(physical_bcis)-seen_bcis)} extra={sorted(seen_bcis-set(physical_bcis))}')
        by_key[key] = method
    target = by_key[('nestedFin', '(I)I')]
    target_report = target['outcome']['report']
    if require_nested_structured:
        need(target_report.get('quality') == 'structured', f'{profile}: nestedFin is not structured')
        need(target_report.get('representation') == 'java' and target_report.get('outcome') == 'produced'
             and target_report.get('fallbacks') == [] and '@bytecode' not in target_report['text'],
             f'{profile}: nestedFin still uses fallback or partial output')
    return by_key


def validate_nested_fixture_document(doc: dict, class_facts, class_length: int, profile: str):
    expected = [('<init>', '()V'), ('nestedFin', '(I)I')]
    methods = doc.get('methods')
    need(isinstance(methods, list) and len(methods) == 2,
         f'{profile}: fixture must contain exactly constructor and nestedFin')
    need([method_key(m) for m in methods] == expected,
         f'{profile}: fixture method ordinal/order/identity differs')
    need([m['item'].get('index') for m in methods] == [0, 1],
         f'{profile}: fixture ordinals are not exactly 0 and 1')
    class_bytes = doc.get('class', {}).get('class_bytes', {})
    need(class_bytes.get('length') == class_length,
         f'{profile}: class identity length differs from pinned class file')
    by_key = {}
    for method, key in zip(methods, expected):
        name, desc = key
        identity = method['item']['identity']
        report = method.get('outcome', {}).get('report')
        need(isinstance(report, dict), f'{profile} {name}{desc}: report missing')
        binding = report.get('artifact', {}).get('binding', {})
        need(report.get('method') == name + desc,
             f'{profile} {name}{desc}: report method signature differs')
        need(binding.get('method') == identity and binding.get('member_ordinal') == method['item']['index'],
             f'{profile} {name}{desc}: physical artifact binding differs')
        need(identity['owner']['class_bytes'] == class_bytes,
             f'{profile} {name}{desc}: method owner differs from class definition')
        physical_bcis = class_facts[key]['instruction_bcis']
        source_map = report.get('source_map', {}).get('segments')
        need(isinstance(source_map, list), f'{profile} {name}{desc}: source map missing')
        report_text = report.get('text')
        need(isinstance(report_text, str), f'{profile} {name}{desc}: report.text missing')
        text_bytes = len(report_text.encode('utf-8'))
        seen_bcis = set()
        for segment in source_map:
            start, end = segment.get('start'), segment.get('end')
            need(isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= text_bytes,
                 f'{profile} {name}{desc}: source-map span outside report.text UTF-8 bytes')
            origin = segment.get('origin', {})
            need(isinstance(origin.get('primary'), dict), f'{profile} {name}{desc}: malformed source origin')
            for item in [origin['primary'], *origin.get('derived', [])]:
                need(item.get('method') == identity,
                     f'{profile} {name}{desc}: origin is bound to a different method')
                bci = item.get('bci')
                need(isinstance(bci, int) and bci in physical_bcis,
                     f'{profile} {name}{desc}: nonphysical source-map BCI {bci}')
                seen_bcis.add(bci)
        need(seen_bcis == set(physical_bcis),
             f'{profile} {name}{desc}: source map misses physical BCIs '
             f'{sorted(set(physical_bcis)-seen_bcis)}')
        by_key[key] = method
    target_report = by_key[('nestedFin', '(I)I')]['outcome']['report']
    need(target_report.get('quality') == 'structured' and target_report.get('representation') == 'java'
         and target_report.get('outcome') == 'produced' and target_report.get('fallbacks') == []
         and '@bytecode' not in target_report['text'],
         f'{profile}: nestedFin is not structured source')
    return by_key


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--out', type=Path, required=True, help='compare.py output directory')
    ap.add_argument('--ex-class', type=Path, required=True, help='sealed EX.class used by compare.py')
    ap.add_argument('--baseline-v2-capture', type=Path, required=True,
                    help='pinned baseline-matrix-root-v2/capture.tar.gz')
    ap.add_argument('--compare-py', type=Path, required=True,
                    help='the exact tools/compare.py used for this run')
    args = ap.parse_args()
    out = args.out.resolve(strict=True)
    execution = read_json(out / 'execution.json')
    root = Path(execution['root']).resolve(strict=True)
    need(execution.get('status') == 'complete_pending_review', 'comparison execution is not complete')
    need(execution.get('expectation') == 'recovered', 'comparison was not run in recovered mode')
    need(execution.get('source_pins_unchanged') is True
         and execution.get('inputs_and_tools_unchanged') is True
         and execution.get('repository_target_absent_after') is True,
         'comparison source/tool/target guards did not all pass')
    commands = {row['index']: row for row in execution.get('commands', [])}
    need(len(commands) == len(execution.get('commands', [])), 'duplicate command indices')
    # Pin all command streams, including setup, compiler, javap, JADX and source-generation rows.
    raw_stream_count = 0
    for row in commands.values():
        need(row.get('guard_stop') is None, f'command {row["index"]} has a guard stop')
        for channel in ('stdout', 'stderr'):
            command_stream(commands, row['index'], channel, root)
            raw_stream_count += 1
    for leg in INPUTS:
        origin = execution.get('origins', {}).get(leg)
        need(origin is not None and origin.get('compiled_class') is not None
             and origin['compiled_class']['sha256'] == origin['input_class']['sha256'],
             f'{leg}: untouched source did not reproduce the committed class bytes')
        need(origin.get('matches_nested_finally_shape') is True,
             f'{leg}: input class lost the independent nested-finally row shape')
        for runtime in RUNTIMES:
            oracle = execution['original_runtime_oracles'][leg][runtime]
            stdout = command_stream(commands, oracle['command'], 'stdout', Path(execution['root']))
            check_18_case_stdout(stdout, f'original {leg}/{runtime}')
            need(hashlib.sha256(stdout).hexdigest() == oracle['stdout_sha256'],
                 f'original {leg}/{runtime}: oracle stdout pin mismatch')
            stderr = command_stream(commands, oracle['command'], 'stderr', Path(execution['root']))
            need(hashlib.sha256(stderr).hexdigest() == oracle['stderr_sha256'],
                 f'original {leg}/{runtime}: oracle stderr pin mismatch')

    spec = importlib.util.spec_from_file_location('nested_fin_compare', args.compare_py.resolve(strict=True))
    need(spec is not None and spec.loader is not None, 'cannot import compare.py parser')
    compare = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(compare)
    method_order = classfile_method_order(args.ex_class)
    need(method_order == METHODS, f'sealed EX method order differs: {method_order}')
    need(sha(args.ex_class) == SEALED_EX_SHA256, 'sealed EX SHA-256 differs from pinned evidence')
    ex_input_pins = [pin for path, pin in execution.get('inputs_before', {}).items()
                     if path.endswith('/fixture/EX.class')]
    need(len(ex_input_pins) == 1 and ex_input_pins[0].get('sha256') == SEALED_EX_SHA256
         and ex_input_pins[0].get('bytes') == args.ex_class.stat().st_size,
         'execution input pin does not bind the sealed EX.class bytes')
    class_facts = {'length': args.ex_class.stat().st_size, 'methods': {}}
    for name, desc in method_order:
        facts = compare.classfile_method_facts(args.ex_class, name, desc)
        class_facts['methods'][(name, desc)] = facts

    # Four independent input/evidence profiles; each compiles under both javacs.
    matrix = execution.get('matrix', [])
    need(len(matrix) == 4, f'expected four full-source profiles, got {len(matrix)}')
    by_profile = {}
    rebuilt_runs = []
    for cell in matrix:
        key = (cell.get('input_compiler'), cell.get('evidence'))
        need(key in {(a, b) for a in INPUTS for b in EVIDENCE}, f'unexpected profile {key}')
        need(key not in by_profile, f'duplicate profile {key}')
        by_profile[key] = cell
        need(cell.get('source') is not None and len(cell.get('compiles', [])) == 2,
             f'{key}: missing source or compiler output')
        need(cell.get('default_all_text_matches') is True, f'{key}: default/all source differs')
        for compiled in cell['compiles']:
            need(compiled.get('exit_code') == 0 and compiled.get('class') is not None,
                 f'{key}/{compiled.get("compiler")}: full source did not compile')
            need(compiled.get('shape', {}).get('matches_nested_catch_all_shape') is True,
                 f'{key}/{compiled.get("compiler")}: nested catch-all rows were not retained')
            comparisons = compiled.get('runtime_comparisons', [])
            need(len(comparisons) == 2, f'{key}/{compiled.get("compiler")}: requires two runtimes')
            for comparison in comparisons:
                runtime = comparison['runtime']
                need(runtime in RUNTIMES and comparison.get('all_match_original') is True
                     and comparison.get('driver_output_valid') is True,
                     f'{key}/{compiled.get("compiler")}/{runtime}: runtime oracle mismatch')
                command_id = comparison['command']
                stdout = command_stream(commands, command_id, 'stdout', root)
                check_18_case_stdout(stdout, f'{key}/{compiled.get("compiler")}/{runtime}')
                oracle = execution['original_runtime_oracles'][key[0]][runtime]
                need(hashlib.sha256(stdout).hexdigest() == oracle['stdout_sha256'],
                     f'{key}/{compiled.get("compiler")}/{runtime}: stdout differs from original')
                stderr = command_stream(commands, command_id, 'stderr', root)
                need(hashlib.sha256(stderr).hexdigest() == oracle['stderr_sha256'],
                     f'{key}/{compiled.get("compiler")}/{runtime}: stderr differs from original')
                rebuilt_runs.append(comparison)
    need(set(by_profile) == {(a, b) for a in INPUTS for b in EVIDENCE}, 'full-source profile matrix incomplete')
    need(sum(len(cell['compiles']) for cell in matrix) == 8 and len(rebuilt_runs) == 16,
         'expected eight successful whole-source compiles and sixteen rebuilt runtime runs')

    # Validate both physical methods and every source-map BCI for each independent compiler input.
    nested_documents = {}
    nested_identity_digests = {}
    for leg in INPUTS:
        origin = execution['origins'][leg]
        class_path = Path(origin['input_class']['path'])
        need(sha(class_path) == origin['input_class']['sha256'], f'{leg}: input class bytes changed')
        fixture_facts = {key: compare.classfile_method_facts(class_path, *key)
                         for key in (('<init>', '()V'), ('nestedFin', '(I)I'))}
        for evidence in EVIDENCE:
            cell = by_profile[(leg, evidence)]
            doc, _ = source_document(out, commands, cell['class_source_command'])
            nested_documents[(leg, evidence)] = doc
            nested_identity_digests[(leg, evidence)] = doc['class']['class_bytes']['digest']
            validate_nested_fixture_document(doc, fixture_facts, class_path.stat().st_size,
                                             f'{leg}/{evidence}')
    for leg in INPUTS:
        need(nested_identity_digests[(leg, 'default')] == nested_identity_digests[(leg, 'all')],
             f'{leg}: default/all class physical identity changed')
        need(nested_documents[(leg, 'default')]['text'] == nested_documents[(leg, 'all')]['text'],
             f'{leg}: default/all Java source text differs')

    # Sealed EX: six physical methods in default/all, stable non-target method reports,
    # bound source-map byte ranges and every physical instruction BCI represented.
    ex_json = {}
    ex_class_identity_digest = None
    ex_docs = {}
    ex_runtime_count = 0
    for evidence in EVIDENCE:
        generated = next((row for row in execution.get('ex_controls', [])
                          if row.get('kind') == 'generated-full-class' and row.get('evidence') == evidence), None)
        need(generated is not None and generated.get('source_generation_exit_code') == 0
             and generated.get('source') is not None, f'sealed EX {evidence}: source missing')
        class_source_command = generated.get('class_source_command')
        doc, raw = source_document(out, commands, class_source_command)
        ex_json[evidence] = doc.get('text')
        need(isinstance(ex_json[evidence], str), f'sealed EX {evidence}: full source text missing')
        class_digest = doc['class']['class_bytes']['digest']
        if ex_class_identity_digest is None:
            ex_class_identity_digest = class_digest
        need(class_digest == ex_class_identity_digest,
             f'sealed EX {evidence}: physical class identity digest differs')
        ex_docs[evidence] = validate_ex_document(doc, class_facts, method_order, f'EX/{evidence}')
        compiles = generated.get('compiles', [])
        need(len(compiles) == 2, f'EX/{evidence}: requires two compiler outputs')
        for compiled in compiles:
            need(compiled.get('exit_code') == 0 and compiled.get('compiled_class') is not None,
                 f'EX/{evidence}/{compiled.get("compiler")}: full class did not compile')
            comparisons = compiled.get('runtime_comparisons', [])
            need(len(comparisons) == 2, f'EX/{evidence}/{compiled.get("compiler")}: requires two runtimes')
            for comparison in comparisons:
                runtime = comparison['runtime']
                build_leg = compiled['compiler']
                need(comparison.get('all_match_original') is True and comparison.get('driver_output_valid') is True,
                     f'EX/{evidence}/{build_leg}/{runtime}: runtime oracle mismatch')
                stdout = command_stream(commands, comparison['command'], 'stdout', root)
                check_18_case_stdout(stdout, f'EX/{evidence}/{build_leg}/{runtime}')
                oracle = execution['ex_original_runtime_oracles'][build_leg][runtime]
                need(hashlib.sha256(stdout).hexdigest() == oracle['stdout_sha256'],
                     f'EX/{evidence}/{build_leg}/{runtime}: stdout differs from original')
                stderr = command_stream(commands, comparison['command'], 'stderr', root)
                need(hashlib.sha256(stderr).hexdigest() == oracle['stderr_sha256'],
                     f'EX/{evidence}/{build_leg}/{runtime}: stderr differs from original')
                ex_runtime_count += 1
    need(ex_json['default'] == ex_json['all'], 'sealed EX default/all Java source text differs')
    need(ex_docs['default'][('nestedFin', '(I)I')]['outcome']['report']['text']
         == ex_docs['all'][('nestedFin', '(I)I')]['outcome']['report']['text'],
         'nestedFin report.text differs across profiles')
    need(len(ex_json) == 2 and sum(len(row['compiles']) for row in execution['ex_controls']
                                   if row.get('kind') == 'generated-full-class') == 4,
         'sealed EX must have two evidence profiles and four whole-source compiler outputs')
    need(ex_runtime_count == 8, f'expected exactly eight sealed EX rebuilt runtime runs, got {ex_runtime_count}')

    ex_shape = next((row for row in execution.get('ex_controls', [])
                     if row.get('kind') == 'sealed-class-shape'), None)
    need(ex_shape is not None and ex_shape.get('matches_nested_finally_shape') is True,
         'sealed EX physical control is absent or changed')
    shape = ex_shape['shape']
    need(shape.get('code_length_bytes') == 72
         and shape.get('classfile_instruction_bcis') ==
         [0, 2, 3, 4, 5, 8, 10, 13, 16, 18, 21, 24, 25, 28, 30, 33, 34, 35, 36, 37,
          38, 41, 43, 46, 49, 51, 54, 57, 59, 62, 64, 67, 69, 70, 71]
         and shape.get('classfile_exception_rows') ==
         [[5, 13, 24, 0], [0, 5, 37, 0], [38, 46, 57, 0], [57, 59, 57, 0]],
         'sealed EX nestedFin Code/exception rows/BCI facts changed')
    for build_leg in RUNTIMES:
        for runtime in RUNTIMES:
            oracle = execution['ex_original_runtime_oracles'][build_leg][runtime]
            stdout = command_stream(commands, oracle['command'], 'stdout', root)
            check_18_case_stdout(stdout, f'original EX/{build_leg}/{runtime}')
            need(hashlib.sha256(stdout).hexdigest() == oracle['stdout_sha256'],
                 f'original EX/{build_leg}/{runtime}: oracle stdout pin mismatch')
            stderr = command_stream(commands, oracle['command'], 'stderr', root)
            need(hashlib.sha256(stderr).hexdigest() == oracle['stderr_sha256'],
                 f'original EX/{build_leg}/{runtime}: oracle stderr pin mismatch')

    with tarfile.open(args.baseline_v2_capture, 'r:gz') as archive:
        need(sha(args.baseline_v2_capture) == BASELINE_SHA256, 'baseline-v2 capture SHA-256 mismatch')
        baseline_docs = {}
        for evidence, member in BASELINE_MEMBERS.items():
            payload = archive.extractfile(member)
            need(payload is not None, f'baseline-v2 capture lacks {member}')
            baseline_docs[evidence] = json.loads(payload.read().decode('utf-8'))
    for evidence in EVIDENCE:
        need(baseline_docs[evidence]['class']['class_bytes']['digest'] == ex_class_identity_digest
             and baseline_docs[evidence]['class']['class_bytes']['length'] == class_facts['length'],
             f'baseline-v2 EX/{evidence}: class identity does not bind the same sealed input')
        baseline = validate_ex_document(baseline_docs[evidence], class_facts, method_order,
                                        f'baseline-v2 EX/{evidence}',
                                        require_nested_structured=False,
                                        require_bci_coverage=False)
        for key in METHODS:
            if key == ('nestedFin', '(I)I'):
                continue
            new_report = ex_docs[evidence][key]['outcome']['report']['text']
            old_report = baseline[key]['outcome']['report']['text']
            need(new_report == old_report,
                 f'EX/{evidence} non-nested method report.text changed: {key[0]}{key[1]}')
    # JADX is a pinned red comparator here: compile all three emissions, preserve the known
    # duplicate-cleanup failure without treating it as a recovered-source gate.
    need(execution.get('jadx_version_verified') is True, 'JADX 1.5.6 version pin was not verified')
    jadx_rows = execution.get('jadx_runs', [])
    need(isinstance(jadx_rows, list) and {row.get('input') for row in jadx_rows} ==
         {'javac8', 'javac23', 'sealed-ex'}, 'JADX input records are incomplete')
    jadx_compiles = 0
    jadx_runtime_rows = []
    for record in jadx_rows:
        need(record.get('exit_code') == 0 and record.get('source') is not None,
             f'JADX source generation failed for {record.get("input")}')
        need(len(record.get('compiles', [])) == 2, f'JADX {record.get("input")}: expected two javac builds')
        for compiled in record['compiles']:
            need(compiled.get('exit_code') == 0 and compiled.get('class') is not None,
                 f'JADX {record["input"]}/{compiled.get("compiler")}: emitted source did not compile')
            jadx_compiles += 1
            for comparison in compiled.get('runtime_comparisons', []):
                command_id = comparison['command']
                stdout = command_stream(commands, command_id, 'stdout', root)
                stderr = command_stream(commands, command_id, 'stderr', root)
                stdout_lines = stdout.decode('utf-8', errors='strict').splitlines()
                stderr_lines = stderr.decode('utf-8', errors='strict').splitlines()
                need(commands[command_id].get('exit_code') == 1
                     and stdout_lines == ['n=2,in=0,out=0: return=5:events=IO']
                     and stderr_lines
                     and stderr_lines[0] ==
                     'Exception in thread "main" java.lang.AssertionError: n=2,in=0,out=1: events=IOIO',
                     f'JADX {record["input"]}/{compiled["compiler"]}/{comparison["runtime"]}: '
                     'pinned duplicate-cleanup control changed')
                need(comparison.get('all_match_original') is False
                     and comparison.get('driver_output_valid') is False,
                     f'JADX {record["input"]}/{compiled["compiler"]}: control classification differs')
                jadx_runtime_rows.append(comparison)
    need(jadx_compiles == 6 and len(jadx_runtime_rows) == 12,
         f'JADX control matrix is incomplete: compiles={jadx_compiles}, runs={len(jadx_runtime_rows)}')
    print(json.dumps({'status': 'accepted', 'full_source_profiles': 4,
                      'whole_source_compiles': 8, 'rebuilt_18_case_runs': 16,
                      'sealed_ex_profiles': 2, 'sealed_ex_methods_per_profile': 6,
                      'sealed_ex_whole_source_compiles': 4,
                      'sealed_ex_rebuilt_18_case_runs': 8,
                      'source_map_method_profiles': 20,
                      'raw_streams_verified': raw_stream_count,
                      'jadx_control_compiles': jadx_compiles,
                      'jadx_pinned_duplicate_cleanup_runs': len(jadx_runtime_rows),
                      'non_nested_ex_reports_match_baseline_v2': True}, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as error:
        raise SystemExit(f'acceptance failed: {type(error).__name__}: {error}')
