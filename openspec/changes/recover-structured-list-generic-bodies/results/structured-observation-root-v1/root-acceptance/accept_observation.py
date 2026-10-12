#!/usr/bin/env python3
"""Verify the saved CF11 diagnostic observation without rebuilding or editing product sources."""
from __future__ import annotations

import ast
import hashlib
import importlib.util
import json
import os
import sys
from pathlib import Path

sys.dont_write_bytecode = True

PREP = Path('/private/tmp/jarde-cf11-structured-void-generic-diag-prep-luna-v3')
RUN_ROOT = Path('/private/tmp/jarde-cf11-structured-void-generic-diagnostic-run-root-v3')
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RECEIPT = RUN_ROOT / 'execution.json'
RESULT = Path(__file__).resolve().parent / 'observation.json'
EXPECTED_TEST = 'nested_iterators_project_and_preserve_the_outer_assignment_and_cast'
EXPECTED_ENV_KEY = 'JARDE_METHOD_SIGNATURE_DIAGNOSTIC'
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
EXPECTED_ATTEMPTS = (1, 2, 3)
EXPECTED_FACT_KEYS = {
    'code_operands', 'code_handlers', 'code_stop', 'code_facts',
    'code_last', 'operation_last', 'program_tree', 'program_last_root',
    'parameter_types', 'parameter_accesses', 'instruction_census',
    'code_bci_set', 'ssa_only_rows', 'effects_only_rows', 'operations', 'ssa_table',
}


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def field_start(text: str, pos: int) -> tuple[str, int] | None:
    while pos < len(text) and text[pos].isspace():
        pos += 1
    start = pos
    if pos >= len(text) or not (text[pos].isalpha() or text[pos] == '_'):
        return None
    pos += 1
    while pos < len(text) and (text[pos].isalnum() or text[pos] == '_'):
        pos += 1
    if pos >= len(text) or text[pos] != '=':
        return None
    return text[start:pos], pos + 1


def parse_top_fields(text: str) -> dict[str, str]:
    """Split whitespace-separated key=value fields outside balanced Rust Debug values."""
    result: dict[str, str] = {}
    pos = 0
    while pos < len(text):
        found = field_start(text, pos)
        need(found is not None, f'malformed top-level field near {text[pos:pos+80]!r}')
        key, value_start = found
        need(key not in result, f'duplicate top-level key {key}')
        stack: list[str] = []
        quote = False
        escaped = False
        i = value_start
        while i < len(text):
            char = text[i]
            if quote:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == '"':
                    quote = False
            else:
                if char == '"':
                    quote = True
                elif char in '([{':
                    stack.append(char)
                elif char in ')]}':
                    need(bool(stack), f'unbalanced closing delimiter in {key}')
                    opener = stack.pop()
                    need({'(': ')', '[': ']', '{': '}'}[opener] == char,
                         f'mismatched delimiter in {key}')
                elif char.isspace() and not stack:
                    next_field = field_start(text, i)
                    if next_field is not None:
                        break
            i += 1
        need(not quote and not stack, f'unclosed value for {key}')
        result[key] = text[value_start:i].strip()
        pos = i
    return result


def split_balanced(text: str, delimiter: str = ',') -> list[str]:
    result: list[str] = []
    stack: list[str] = []
    quote = False
    escaped = False
    start = 0
    for i, char in enumerate(text):
        if quote:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == '"':
                quote = False
        else:
            if char == '"':
                quote = True
            elif char in '([{':
                stack.append(char)
            elif char in ')]}':
                need(bool(stack), 'unbalanced closing delimiter')
                opener = stack.pop()
                need({'(': ')', '[': ']', '{': '}'}[opener] == char,
                     'mismatched delimiter')
            elif char == delimiter and not stack:
                result.append(text[start:i].strip())
                start = i + 1
    need(not quote and not stack, 'unclosed balanced value')
    tail = text[start:].strip()
    if tail:
        result.append(tail)
    return result


def closing_delimiter(text: str, open_index: int) -> int:
    pairs = {'(': ')', '[': ']', '{': '}'}
    opener = text[open_index]
    need(opener in pairs, f'not an opening delimiter at {open_index}')
    stack: list[str] = []
    quote = False
    escaped = False
    for i in range(open_index, len(text)):
        char = text[i]
        if quote:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == '"':
                quote = False
        else:
            if char == '"':
                quote = True
            elif char in '([{':
                stack.append(char)
            elif char in ')]}':
                need(bool(stack), 'unbalanced closing delimiter')
                active = stack.pop()
                need(pairs[active] == char, 'mismatched delimiter')
                if not stack:
                    return i
    raise ValueError('unterminated balanced value')


def parse_list(text: str) -> list[str]:
    text = text.strip()
    need(text.startswith('[') and text.endswith(']'), f'expected Rust list, got {text[:80]!r}')
    body = text[1:-1].strip()
    return split_balanced(body) if body else []


def parse_tuple(text: str) -> list[str]:
    text = text.strip()
    need(text.startswith('(') and text.endswith(')'), f'expected Rust tuple, got {text[:80]!r}')
    return split_balanced(text[1:-1])


def parse_struct(text: str, name: str) -> tuple[dict[str, str], int, int]:
    text = text.strip()
    prefix = name + ' {'
    need(text.startswith(prefix), f'expected {name} struct')
    open_index = len(name) + 1
    close_index = closing_delimiter(text, open_index)
    need(not text[close_index + 1:].strip(), f'trailing bytes after {name}')
    body = text[open_index + 1:close_index]
    fields: dict[str, str] = {}
    for item in split_balanced(body):
        pieces = split_balanced(item, ':')
        need(len(pieces) >= 2, f'malformed {name} field {item[:80]!r}')
        key = pieces[0]
        value = ':'.join(pieces[1:]).strip()
        need(key not in fields, f'duplicate {name} field {key}')
        fields[key] = value
    return fields, open_index, close_index


def parse_some(text: str) -> str:
    text = text.strip()
    need(text.startswith('Some(') and text.endswith(')'), f'expected Some(...), got {text[:80]!r}')
    close = closing_delimiter(text, 4)
    need(close == len(text) - 1, 'trailing bytes after Some(...)')
    return text[5:-1].strip()


def parse_local_slot(text: str) -> int | None:
    text = text.strip()
    if not text.startswith('Local('):
        return None
    close = closing_delimiter(text, 5)
    need(close == len(text) - 1, 'malformed Local slot')
    return int(text[6:close].strip())


def extract_snapshot(code_facts: str) -> tuple[dict[str, int], str]:
    marker = 'UsageSnapshot {'
    starts: list[int] = []
    pos = 0
    while True:
        pos = code_facts.find(marker, pos)
        if pos < 0:
            break
        starts.append(pos)
        pos += len(marker)
    need(len(starts) == 1, f'expected one UsageSnapshot, found {len(starts)}')
    start = starts[0]
    open_index = start + len(marker) - 1
    close_index = closing_delimiter(code_facts, open_index)
    body = code_facts[open_index + 1:close_index]
    values: dict[str, int] = {}
    for item in split_balanced(body):
        pieces = split_balanced(item, ':')
        need(len(pieces) == 2, f'malformed usage counter {item!r}')
        key, raw = pieces[0], pieces[1]
        need(key not in values, f'duplicate usage counter {key}')
        values[key] = int(raw)
    normalized = code_facts[:start] + 'UsageSnapshot { <budget-snapshot> }' + code_facts[close_index + 1:]
    return values, normalized


def event_records(stderr: str) -> dict[str, dict[int, dict[str, str]]]:
    found: dict[str, dict[int, dict[str, str]]] = {}
    for line in stderr.splitlines():
        if not line.startswith('JARDE_METHOD_SIGNATURE event='):
            continue
        fields = parse_top_fields(line[len('JARDE_METHOD_SIGNATURE '):])
        event = fields.pop('event')
        attempt = int(fields.pop('attempt'))
        need(attempt not in found.setdefault(event, {}), f'duplicate {event} attempt {attempt}')
        found[event][attempt] = fields
    return found


def expression_nodes(text: str, prefix: str) -> list[str]:
    results: list[str] = []
    pos = 0
    while True:
        start = text.find(prefix, pos)
        if start < 0:
            break
        open_index = text.find('{', start)
        close_index = closing_delimiter(text, open_index)
        results.append(text[start:close_index + 1])
        pos = start + len(prefix)
    return results


def origin_bcis(expr_fields: dict[str, str]) -> list[int]:
    origin_set, _, _ = parse_struct(expr_fields['origin'], 'OriginSet')
    origins = [origin_set['primary'], *parse_list(origin_set['derived'])]
    result = []
    for origin in origins:
        fields, _, _ = parse_struct(origin, 'Origin')
        result.append(int(fields['bci']))
    return result


def source_closure(runner_source: str) -> tuple[dict[str, str], dict[str, str]]:
    tree = ast.parse(runner_source)
    constants: dict[str, ast.expr] = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            constants[node.targets[0].id] = node.value

    def path_constant(name: str) -> Path:
        value = constants[name]
        need(isinstance(value, ast.Call) and isinstance(value.func, ast.Name) and value.func.id == 'Path',
             f'{name} is not the pinned Path literal')
        args = ast.literal_eval(value.args[0])
        return Path(args)

    closure_path = path_constant('CLOSURE')
    closure_sha = ast.literal_eval(constants['CLOSURE_SHA'])
    need(sha_file(closure_path) == closure_sha, 'dynamic source-closure helper SHA mismatch')
    spec = importlib.util.spec_from_file_location('cf11_observation_closure', closure_path)
    need(spec is not None and spec.loader is not None, 'cannot load pinned source-closure helper')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    paths = module.source_paths()
    expected_keys = {str(path.relative_to(module.ROOT)) for path in paths}
    live_pins = module.pin_sources(paths)
    need(set(live_pins) == expected_keys, 'closure helper Path/key mismatch')
    return live_pins, {'path': str(closure_path), 'sha256': closure_sha}


def pinned_path_and_sha(tree: ast.Module, path_name: str, sha_name: str) -> tuple[Path, str]:
    values: dict[str, ast.expr] = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            values[node.targets[0].id] = node.value
    path_value = values[path_name]
    if isinstance(path_value, ast.Call) and isinstance(path_value.func, ast.Name) and path_value.func.id == 'Path':
        path = Path(ast.literal_eval(path_value.args[0]))
    elif (isinstance(path_value, ast.BinOp) and isinstance(path_value.op, ast.Div) and
          isinstance(path_value.left, ast.Name) and path_value.left.id == 'ROOT'):
        path = ROOT / ast.literal_eval(path_value.right)
    else:
        raise ValueError(f'{path_name} is not a pinned Path expression')
    return path, ast.literal_eval(values[sha_name])


def verify_guard_sampler(guard_path: Path) -> None:
    tree = ast.parse(guard_path.read_text())
    run_command = next(node for node in tree.body
                       if isinstance(node, ast.FunctionDef) and node.name == 'run_command')
    sleeps_one_second = any(
        isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and
        isinstance(node.func.value, ast.Name) and node.func.value.id == 'time' and
        node.func.attr == 'sleep' and len(node.args) == 1 and
        isinstance(node.args[0], ast.Constant) and node.args[0].value == 1
        for node in ast.walk(run_command)
    )
    need(sleeps_one_second, 'pinned guard run_command does not sample once per second')


def verify() -> dict:
    pins = json.loads((PREP / 'pins.json').read_text())
    runner_path = PREP / 'run_diagnostic.py'
    patch_path = PREP / 'diagnostic.patch'
    runner_source = runner_path.read_text()
    runner_tree = ast.parse(runner_source)
    need(sha_file(runner_path) == 'ad947cdc0082278cba7903b593a2ff6e6b0724b46c711c799a01c079025b4555',
         'diagnostic runner SHA mismatch')
    need(sha_file(patch_path) == pins['artifacts_sha256']['diagnostic.patch'] ==
         'cf9ad80f96b20949b505f047871652f562a4540f9c76115175dc94feba6e2665',
         'diagnostic patch SHA mismatch')
    for name in ('run_diagnostic.py', 'runplan.md', 'README.md'):
        need(sha_file(PREP / name) == pins['artifacts_sha256'][name], f'prep artifact pin mismatch: {name}')
    guard_path, guard_sha = pinned_path_and_sha(runner_tree, 'GUARD', 'GUARD_SHA')
    scan_path, scan_sha = pinned_path_and_sha(runner_tree, 'SCAN', 'SCAN_SHA')
    need(sha_file(guard_path) == guard_sha and sha_file(scan_path) == scan_sha,
         'pinned cleanup guard or target-size scanner SHA mismatch')
    verify_guard_sampler(guard_path)

    data = json.loads(RECEIPT.read_text())
    receipt_sha = sha_file(RECEIPT)
    need(data.get('schema') == 'cf11-method-signature-diagnostic-observation-v1', 'unexpected execution schema')
    need(data.get('status') == 'failed' and
         data.get('error') == "AssertionError('detailed Code/Program/SSA facts differ by request profile')",
         'receipt is not the known byte-for-byte consistency-assertion failure')
    need(data.get('runner', {}).get('path') == str(runner_path) and
         data['runner'].get('sha256_before') == data['runner'].get('sha256_after') == sha_file(runner_path),
         'runner path or before/after SHA mismatch')
    need(data.get('patch') == {'path': str(patch_path), 'sha256': sha_file(patch_path)},
         'receipt does not bind the diagnostic patch')
    need(data.get('guard_limits') == {'minimum_free_bytes': FREE_LIMIT, 'maximum_target_bytes': TARGET_LIMIT},
         'recorded test guard limits differ')

    files = {
        'src/class_source.rs': ('class_source.rs.base', 'class_source.rs.candidate'),
        'crates/jarde-java/src/report.rs': ('report.rs.base', 'report.rs.candidate'),
    }
    for rel, (base_name, candidate_name) in files.items():
        base_sha = pins['source_base_sha256'][rel]
        candidate_sha = pins['source_candidate_sha256'][rel]
        need(sha_file(PREP / base_name) == base_sha, f'prepared base sample SHA mismatch: {rel}')
        need(sha_file(PREP / candidate_name) == candidate_sha, f'prepared candidate sample SHA mismatch: {rel}')
        need(data['base_files'][rel]['sha256'] == base_sha, f'receipt base pin mismatch: {rel}')
        need(data['candidate_files'][rel]['sha256'] == candidate_sha, f'receipt candidate pin mismatch: {rel}')
        need(sha_file(ROOT / rel) == base_sha, f'live source is not restored base bytes: {rel}')
    need(set(data.get('restored_files', {})) == set(files) and
         all(row.get('exact_base_bytes_restored') is True and
             row.get('sha256') == data['base_files'][rel]['sha256']
             for rel, row in data['restored_files'].items()),
         'exact source-byte restore receipt mismatch')

    fixture = data['fixture']
    need(fixture['path'] == 'openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class' and
         fixture['sha256'] == pins['fixture_sha256'] and
         sha_file(ROOT / fixture['path']) == fixture['sha256'] and
         fixture.get('present_in_dynamic_source_closure') is True,
         'fixed fixture identity mismatch')

    live_pins, closure = source_closure(runner_source)
    before = data.get('source_pins_before', {})
    during = data.get('source_pins_during', {})
    after = data.get('source_pins_after', {})
    need(len(before) > 0 and len(before) == len(during) == len(after),
         'recorded dynamic closure is empty or has changing counts')
    need(data.get('source_pin_count_before') == data.get('source_pin_count_after') == len(before),
         'recorded source-pin counts do not match maps')
    need(set(before) == set(during) == set(after) == set(live_pins), 'dynamic closure key sets differ')
    need(live_pins == before == after and data.get('source_pins_unchanged') is True,
         'live/before/after source pins do not match exactly')
    changed_during = {key for key in before if before[key] != during[key]}
    need(changed_during == set(files), f'during-run changes are not exactly the two injected files: {changed_during}')
    need({key: during[key] for key in changed_during} == pins['source_candidate_sha256'],
         'during-run candidate source hashes differ')

    commands = data.get('commands', [])
    need(len(commands) == 2, 'receipt must contain the diagnostic and final clean commands')
    test, clean = commands
    expected_argv = ['cargo', 'test', '--test', 'p3_hoisted_iterator_foreach', EXPECTED_TEST,
                     '--', '--exact', '--nocapture']
    expected_env = {**{
        'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0',
        'CARGO_PROFILE_TEST_DEBUG': '0', 'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
    }, EXPECTED_ENV_KEY: '1'}
    need(test.get('index') == 0 and test.get('argv') == expected_argv and test.get('cwd') == str(ROOT) and
         test.get('exit_code') == 0 and test.get('guard_stop') is None and
         test.get('env_overrides') == expected_env and
         test.get('peak_target_bytes', TARGET_LIMIT + 1) <= TARGET_LIMIT and
         test.get('free_bytes_after', 0) >= FREE_LIMIT,
         'diagnostic test command/guard receipt mismatch')
    need(data.get('test', {}).get('argv') == expected_argv and
         data['test'].get('requests') == 'existing test performs default, all-evidence, and no-rule-details class-source requests',
         'test request set differs from the pinned existing test')
    need(clean.get('index') == 1 and clean.get('argv') == ['cargo', 'clean'] and clean.get('cwd') == str(ROOT) and
         clean.get('exit_code') == 0 and clean.get('guard_stop') is None and clean.get('cleanup_only') == {
             'exact_argv': ['cargo', 'clean'], 'space_precheck_bypassed': True,
             'space_monitor_abort_bypassed': True,
         } and clean.get('duration_seconds', 0) >= 1 and clean.get('env_overrides') == expected_env and
         clean.get('free_bytes_after', 0) >= FREE_LIMIT,
         'cleanup command/guard receipt mismatch')
    need(data.get('cleanup') == {'attempted': True, 'status': 'passed', 'target_absent': True} and
         data.get('target_absent') is True and not (ROOT / 'target').exists(),
         'target clean receipt or current target state mismatch')

    raw_evidence = {}
    for command in commands:
        for stream_name in ('stdout', 'stderr'):
            stream = command['streams'][stream_name]
            path = Path(stream['path'])
            need(path.parent == RUN_ROOT and path.is_file() and not path.is_symlink(),
                 f'raw stream is missing/out of scope: {path}')
            raw = path.read_bytes()
            need(len(raw) == stream['bytes'] and hashlib.sha256(raw).hexdigest() == stream['sha256'],
                 f'raw stream size/SHA mismatch: {path.name}')
            raw_evidence[path.name] = {'path': str(path), 'bytes': len(raw), 'sha256': sha_file(path)}
    stdout = (RUN_ROOT / '0.stdout.raw').read_text(errors='replace')
    need(f'test {EXPECTED_TEST} ... ok' in stdout and
         'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out;' in stdout,
         'raw test stream does not show the exact named test passing')

    stderr = (RUN_ROOT / '0.stderr.raw').read_text(errors='replace')
    events = event_records(stderr)
    need(set(events) >= {'parsed', 'terminal', 'ssa_input', 'ssa_reject', 'structured_facts'},
         'raw stream is missing required diagnostic event groups')
    for event in ('parsed', 'terminal', 'ssa_input', 'ssa_reject', 'structured_facts'):
        need(set(events[event]) == set(EXPECTED_ATTEMPTS), f'{event} attempts are not exactly 1,2,3')

    structured: dict[int, dict[str, str]] = {}
    counters: dict[int, dict[str, int]] = {}
    normalized: dict[int, dict[str, str]] = {}
    fact_key_sets = {}
    for attempt in EXPECTED_ATTEMPTS:
        row = events['structured_facts'][attempt]
        need(set(row) == EXPECTED_FACT_KEYS, f'structured_facts top-level keys differ: {sorted(row)}')
        fact_key_sets[attempt] = sorted(row)
        need(int(row['code_operands']) == 45, f'attempt {attempt} Code operand count is not 45')
        counters[attempt], normalized_code = extract_snapshot(row['code_facts'])
        normalized[attempt] = {**row, 'attempt': '<attempt>', 'code_facts': normalized_code}
        structured[attempt] = row
    need(counters[1].keys() == counters[2].keys() == counters[3].keys(), 'budget snapshot schemas differ')
    counter_differences = {
        key: {attempt: counters[attempt][key] for attempt in EXPECTED_ATTEMPTS}
        for key in counters[1]
        if len({counters[attempt][key] for attempt in EXPECTED_ATTEMPTS}) > 1
    }
    need(set(counter_differences) <= {'ir_items', 'elapsed_millis'},
         f'non-budget structural counters vary across requests: {counter_differences}')
    baseline = normalized[1]
    structural_differences = {}
    for attempt in EXPECTED_ATTEMPTS[1:]:
        keys = set(baseline) | set(normalized[attempt])
        changed = {key: {'attempt_1': baseline.get(key), f'attempt_{attempt}': normalized[attempt].get(key)}
                   for key in keys if baseline.get(key) != normalized[attempt].get(key)}
        if changed:
            structural_differences[attempt] = changed
    need(not structural_differences,
         f'non-budget structured facts differ across request profiles: {structural_differences}')

    # Independently cross-check the per-BCI census against the complete decoded Code and Operations.
    code_fields, _, _ = parse_struct(structured[1]['code_facts'], 'MethodCodeFacts')
    code_instructions = parse_list(code_fields['instructions'])
    code_operands = parse_list(code_fields['operands'])
    census = [parse_tuple(item) for item in parse_list(structured[1]['instruction_census'])]
    operation_rows = [parse_tuple(item) for item in parse_list(structured[1]['operations'])]
    need(len(code_instructions) == len(code_operands) == len(census) == len(operation_rows) == 45,
         'Code, operands, census, and Operations do not each contain 45 rows')
    census_by_bci = {}
    for row in census:
        need(len(row) == 7, 'instruction census row arity differs')
        bci, raw_opcode, effective_opcode, cp_index = int(row[0]), int(row[1]), int(row[2]), row[3]
        need(bci not in census_by_bci, f'duplicate Code BCI {bci}')
        ssa_rows = [parse_tuple(item) for item in parse_list(row[4])]
        effect_rows = [parse_tuple(item) for item in parse_list(row[5])]
        need(len(ssa_rows) == len(effect_rows) == 1, f'BCI {bci} lacks exactly one SSA/effects row')
        need(len(ssa_rows[0]) == 5 and len(effect_rows[0]) == 6 and
             int(ssa_rows[0][0]) == int(effect_rows[0][0]) == bci,
             f'BCI {bci} SSA/effects rows have wrong shape or identity')
        operation = parse_some(row[6])
        census_by_bci[bci] = {'raw_opcode': raw_opcode, 'effective_opcode': effective_opcode,
                              'cp_index': cp_index, 'ssa': ssa_rows[0],
                              'effects': effect_rows[0], 'operation': operation}
    need(len(census_by_bci) == 45, 'physical Code BCI set is not unique and complete')
    for item in code_instructions:
        fields, _, _ = parse_struct(item, 'InstructionFact')
        bci = int(fields['bci'])
        need(bci in census_by_bci and int(fields['opcode']) == census_by_bci[bci]['raw_opcode'] and
             fields['constant_pool_index'] == census_by_bci[bci]['cp_index'],
             f'Code instruction and census differ at BCI {bci}')
    for (bci_text, operation_text) in operation_rows:
        bci = int(bci_text)
        need(bci in census_by_bci and census_by_bci[bci]['operation'] == operation_text,
             f'Operation list and census differ at BCI {bci}')
    need(len({int(row[0]) for row in operation_rows}) == 45 and
         {int(row[0]) for row in operation_rows} == set(census_by_bci),
         'Operation BCI coverage is not one-to-one with Code')
    bci_set = structured[1]['code_bci_set'].strip()
    need(bci_set.startswith('{') and bci_set.endswith('}') and
         {int(item) for item in split_balanced(bci_set[1:-1])} == set(census_by_bci),
         'recorded Code BCI set differs from census')
    need(structured[1]['ssa_only_rows'] == '[]' and structured[1]['effects_only_rows'] == '[]',
         'SSA/effects contain rows outside the physical Code BCI set')

    # Prove the explicitly printed formal-parameter access list is exactly the census projection.
    parameter_types = structured[1]['parameter_types'].strip()
    need(parameter_types.startswith('{') and parameter_types.endswith('}'), 'parameter type table is not a map')
    parameter_slots = {int(split_balanced(item, ':')[0]) for item in split_balanced(parameter_types[1:-1])}
    need(parameter_slots == {1, 2}, f'unexpected descriptor parameter slots: {parameter_slots}')

    def accesses_for_slot_list(text: str, bci: int) -> tuple[tuple[int, str], ...]:
        accesses = []
        for item in parse_list(text):
            pair = parse_tuple(item)
            need(len(pair) == 2, 'SSA slot/value pair arity differs')
            local_slot = parse_local_slot(pair[0])
            if local_slot in parameter_slots:
                accesses.append((local_slot, pair[1]))
        return tuple(accesses)

    derived_accesses = []
    for bci in sorted(census_by_bci):
        row = census_by_bci[bci]['ssa']
        reads = accesses_for_slot_list(row[3], bci)
        writes = accesses_for_slot_list(row[4], bci)
        if reads or writes:
            derived_accesses.append((bci, row[1], int(row[2]), reads, writes))
    printed_accesses = []
    for item in parse_list(structured[1]['parameter_accesses']):
        row = parse_tuple(item)
        need(len(row) == 5, 'parameter access row arity differs')
        reads = accesses_for_slot_list(row[3], int(row[0]))
        writes = accesses_for_slot_list(row[4], int(row[0]))
        printed_accesses.append((int(row[0]), row[1], int(row[2]), reads, writes))
    need(sorted(derived_accesses, key=lambda row: row[0]) == sorted(printed_accesses, key=lambda row: row[0]),
         'parameter_accesses differs from the full per-BCI SSA reads/writes census')
    need({slot for _, _, _, reads, _ in derived_accesses for slot, _ in reads} == parameter_slots and
         all(not writes for _, _, _, _, writes in derived_accesses),
         'the two descriptor parameters are not both read or show writes')

    # Correlate decoded add/remove targets with their Program AST calls and preserve cast snippets.
    call_targets = {}
    for bci, row in census_by_bci.items():
        operation = row['operation']
        if operation.startswith('Invoke(CallTarget {'):
            call_fields, _, _ = parse_struct(operation[len('Invoke('):-1], 'CallTarget')
            call_targets[bci] = call_fields
    named_targets = {name: [(bci, fields) for bci, fields in call_targets.items()
                            if fields.get('name') == f'"{name}"'] for name in ('add', 'remove')}
    need(all(len(named_targets[name]) == 1 for name in ('add', 'remove')),
         f'expected one add/remove invocation each: { {k:len(v) for k,v in named_targets.items()} }')
    for name, rows in named_targets.items():
        _, target = rows[0]
        need(target.get('owner') == '"java/util/List"' and
             target.get('descriptor') == '"(Ljava/lang/Object;)Z"' and
             target.get('kind') == 'Interface' and target.get('interface_reference') == 'true',
             f'{name}(Object) invocation target differs: {target}')

    tree_text = structured[1]['program_tree']
    call_nodes = []
    for fragment in expression_nodes(tree_text, 'Expr { kind: Call {'):
        expr_fields, _, _ = parse_struct(fragment, 'Expr')
        call_fields, _, _ = parse_struct(expr_fields['kind'], 'Call')
        if call_fields.get('name') in {'"add"', '"remove"'}:
            bcis = origin_bcis(expr_fields)
            target_name = call_fields['name'].strip('"')
            target_bci = named_targets[target_name][0][0]
            need(target_bci in bcis, f'AST {target_name} call origins do not include invocation BCI {target_bci}')
            args = parse_list(call_fields['args'])
            need(len(args) == 1 and args[0].startswith('Expr { kind: Cast {'),
                 f'AST {target_name} call does not preserve one explicit argument conversion')
            call_nodes.append({'name': target_name, 'call_bci': target_bci,
                               'ast_origins': bcis, 'fragment': fragment})
    need({row['name'] for row in call_nodes} == {'add', 'remove'} and len(call_nodes) == 2,
         'Program AST does not contain exactly the two correlated add/remove calls')

    casts = []
    physical_checkcast_bcis = {bci for bci, row in census_by_bci.items()
                               if row['operation'].startswith('CheckCast {')}
    for fragment in expression_nodes(tree_text, 'Expr { kind: Cast {'):
        expr_fields, _, _ = parse_struct(fragment, 'Expr')
        cast_fields, _, _ = parse_struct(expr_fields['kind'], 'Cast')
        bcis = origin_bcis(expr_fields)
        matching = sorted(set(bcis) & physical_checkcast_bcis)
        if matching:
            casts.append({'physical_checkcast_bcis': matching, 'ast_origins': bcis, 'fragment': fragment})
    need({bci for row in casts for bci in row['physical_checkcast_bcis']} == physical_checkcast_bcis and
         len(physical_checkcast_bcis) == 2,
         'physical checkcast operations do not have a complete matching AST Cast set')

    for event_name in ('parsed', 'terminal', 'ssa_input', 'ssa_reject'):
        rows = events[event_name]
        baseline_event = {k: v for k, v in rows[1].items()}
        for attempt in EXPECTED_ATTEMPTS[1:]:
            need(rows[attempt] == baseline_event, f'{event_name} differs at attempt {attempt}')
    need(events['ssa_input'][1].get('code_instructions') == '45' and
         events['ssa_input'][1].get('operation_count') == '45' and
         events['ssa_input'][1].get('ssa_phi_count') == '21' and
         events['ssa_input'][1].get('ssa_blocks') == '13',
         'ssa_input summary differs from the observed complete method census')
    need(all(row.get('state') == 'refused' and row.get('stage') == 'project_method_signature_with_state'
             for row in events['terminal'].values()), 'terminal outcomes are not the same refusal path')
    need(len({row['rejected_line'] for row in events['ssa_reject'].values()}) == 1,
         'first rejection source line differs by attempt')

    normalized_json = json.dumps(normalized, sort_keys=True, separators=(',', ':')).encode()
    output = {
        'schema': 'cf11-method-signature-observation-acceptance-v1',
        'status': 'observation_bundle_verified_for_manual_review',
        'product_acceptance': False,
        'generic_binding_claim': False,
        'execution_receipt': {'path': str(RECEIPT), 'sha256': receipt_sha,
                              'original_status': data['status'], 'original_error': data['error'],
                              'workspace_head_observed': data['workspace_head_observed']},
        'pinned_inputs': {
            'runner': {'path': str(runner_path), 'sha256': sha_file(runner_path)},
            'patch': {'path': str(patch_path), 'sha256': sha_file(patch_path)},
            'closure_helper': closure,
            'base_source_pins': pins['source_base_sha256'],
            'candidate_source_pins': pins['source_candidate_sha256'],
            'fixture': fixture,
        },
        'workspace_source_closure': {'count': len(before), 'live_before_after_equal': True,
                                     'during_only_injected_files_changed': sorted(changed_during)},
        'commands': [{'index': row['index'], 'argv': row['argv'], 'exit_code': row['exit_code'],
                      'guard_stop': row['guard_stop'], 'peak_target_bytes': row['peak_target_bytes'],
                      'free_bytes_after': row['free_bytes_after'],
                      'stdout': row['streams']['stdout'], 'stderr': row['streams']['stderr']}
                     for row in commands],
        'restore_and_cleanup': {'restored_exactly': True, 'cleanup_passed': True,
                                'target_absent': True, 'rust_stack': data['rust_min_stack']},
        'raw_streams': raw_evidence,
        'diagnostic_groups': {'attempts': list(EXPECTED_ATTEMPTS),
                              'events_per_attempt': ['ssa_input', 'structured_facts', 'ssa_reject', 'parsed', 'terminal'],
                              'first_rejection_source_line': int(events['ssa_reject'][1]['rejected_line']),
                              'terminal_state': events['terminal'][1]['state'],
                              'terminal_reason': events['terminal'][1]['error']},
        'structured_fact_keys': fact_key_sets[1],
        'counter_snapshots_by_attempt': {str(attempt): counters[attempt] for attempt in EXPECTED_ATTEMPTS},
        'counter_differences_by_field': counter_differences,
        'non_budget_structural_facts_equal': True,
        'normalized_structural_sha256': hashlib.sha256(normalized_json).hexdigest(),
        'physical_instruction_census': {'code': 45, 'operands': 45, 'operations': 45,
                                        'ssa_rows': 45, 'effects_rows': 45,
                                        'unique_bcis': 45, 'parameter_slots': sorted(parameter_slots),
                                        'parameter_access_rows': len(printed_accesses),
                                        'parameter_slots_read': sorted({slot for _, _, _, reads, _ in derived_accesses
                                                                        for slot, _ in reads}),
                                        'parameter_writes': 0},
        'manual_review_snippets': {'add_remove_ast_calls': call_nodes,
                                   'physical_checkcast_ast_nodes': casts,
                                   'physical_checkcast_bcis': sorted(physical_checkcast_bcis)},
        'review_limits': [
            'The original runner receipt remains failed because its byte-for-byte assertion included resource counters.',
            'This acceptor preserves and reports every UsageSnapshot field; only the snapshot block is excluded from structural equality.',
            'This verifies observation integrity and cross-request structural consistency only; it does not accept a product fix or prove generic binding.',
            'The erased java/util/List.add(Object) reference does not by itself prove the generic E-to-source-type substitution.',
        ],
    }
    RESULT.write_text(json.dumps(output, indent=2, ensure_ascii=False) + '\n')
    return output


if __name__ == '__main__':
    result = verify()
    print(json.dumps({'status': result['status'], 'receipt': str(RESULT),
                      'attempts': result['diagnostic_groups']['attempts'],
                      'counter_differences': result['counter_differences_by_field'],
                      'census': result['physical_instruction_census']}, ensure_ascii=False))
