#!/usr/bin/env python3
"""Bind current-product GitHub CI to the fresh complete compound workspace replay."""
from __future__ import annotations
import argparse
import gzip
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
HELPER_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py'
HELPER_SHA = 'bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
CAPTURE_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py'
CAPTURE_SHA = 'ed889bf6af08247efcce3c889cd1c790d6829d0ea12e9c0d053bdef801994be3'
PUBLIC_LABEL_RULE = Path('/private/tmp/jarde-conditional-ci-public-labels-root-v1.py')
PUBLIC_LABEL_RULE_SHA = 'd7a6bcebb703b1520332cf4c7419c50d21d919d6a0c6123162d0d4d78bbcac65'
CHECKLIST = Path('/private/tmp/jarde-arithmetic-stack-own-ci-verifier-luna-v1/ci-handoff-checklist.md')
CHECKLIST_SHA = 'b7a0a8860ec8347b721bbb59583159d111dab9825178504b93ddfe3ae8d881bf'
BUILD_EVIDENCE = Path('/private/tmp/jarde-arithmetic-stack-build-evidence-luna-v2/build_evidence.py')
BUILD_EVIDENCE_SHA = '9d3c76233f23757522a3edbd52017c3a845f188f34e1086269502d43f43b2afb'
BUILD_RUNNER = Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2.py')
BUILD_RUNNER_SHA = 'dc0c74fa9e5944a2cc19a7968bed6c66e700ba7f1c798cab6d32167f04f24844'
FAILED_WORKSPACE_V3 = Path('/private/tmp/jarde-compound-full-workspace-root-v3').resolve()
WORKSPACE_RUNNER = Path('/private/tmp/jarde-compound-workspace-batches-root-v1.py')
WORKSPACE_RUNNER_SHA = '2ac64ccdf7ed6c8f90f37b8ae34693eb16206cba493ca4a3e261de993a08235b'
BUILD_RECORD = Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2/execution.json')
WORKSPACE_SCHEMA = 'compound-full-workspace-batched-root-v1'
SEEDS = ('5350648285461741569', '5350648285461741570')
WORKSPACE_CMD = 'cargo test --workspace --all-targets --all-features --locked'
TARGET_COUNT = 359
MIN_SOURCE_PINS = 2210
WORKFLOW_SHA = 'b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a'
MASKED_OLD_NULL_TEST = 'null_first_exact_builder_writes_refine_only_the_local_type'
RENAMED_NULL_TEST = 'null_first_exact_builder_writes_refine_local_type_and_compound_condition'

spec = importlib.util.spec_from_file_location('conditional_ci_v5_helpers', HELPER_PATH)
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)
sys.path.insert(0, str(BUILD_EVIDENCE.parent))
from build_evidence import verify_build  # noqa: E402


def need(ok: bool, message: str):
    if not ok:
        raise ValueError(message)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(Path(path).read_bytes())


def load_json(path: Path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def checklist_pins() -> dict[str, str]:
    need(sha_file(CHECKLIST) == CHECKLIST_SHA, 'private current-product CI checklist pin mismatch')
    text = CHECKLIST.read_text(encoding='utf-8')
    rows = {}
    in_table = False
    for line in text.splitlines():
        if line.strip() == '| Path | SHA-256 |':
            in_table = True
            continue
        if in_table and not line.startswith('|'):
            break
        if in_table:
            match = re.fullmatch(r'\| ([^|]+) \| ([0-9a-f]{64}) \|', line.strip())
            if match:
                rows[match.group(1)] = match.group(2)
    need(rows and '.github/workflows/ci.yml' in rows, 'checklist source-pin table missing')
    need(rows['.github/workflows/ci.yml'] == WORKFLOW_SHA, 'checklist workflow pin differs from reviewed workflow')
    return rows


def verify_workspace(product: str, workspace: Path):
    workspace = workspace.resolve(strict=True)
    need(workspace != FAILED_WORKSPACE_V3, 'refusing the known failed workspace-root-v3 evidence')
    need(workspace.is_dir(), 'workspace evidence path must be a directory')
    execution_path = workspace / 'execution.json'
    metadata_path = workspace / 'metadata.json'
    ex_raw, metadata_raw = execution_path.read_bytes(), metadata_path.read_bytes()
    ex, metadata = json.loads(ex_raw), json.loads(metadata_raw)
    ex_sha, metadata_sha = sha(ex_raw), sha(metadata_raw)
    need(ex.get('schema') == WORKSPACE_SCHEMA and ex.get('status') == 'passed',
         'fresh full-workspace execution is incomplete or not passing')
    need(ex.get('seed') == SEEDS[0], 'fresh workspace seed differs from first CI seed')
    need(ex.get('no_carried_execution') is True and ex.get('pins_unchanged') is True,
         'fresh workspace was carried or source pins changed')
    need(ex.get('target_size_scan_adapter') == {'path': str(ci.SCAN), 'sha256': ci.SCAN_SHA},
         'fresh workspace target-size scanner pin mismatch')
    need(ex.get('guard_template') == {'path': str(ci.GUARD), 'sha256': ci.GUARD_SHA},
         'fresh workspace resource guard pin mismatch')
    need(ci.SCAN.is_file() and sha_file(ci.SCAN) == ci.SCAN_SHA and
         sha(ci.git_blob(product, str(ci.SCAN.relative_to(ROOT)))) == CI_SCAN_SHA,
         'target-size scanner is not pinned in the current product commit')
    need(ci.GUARD.is_file() and sha_file(ci.GUARD) == ci.GUARD_SHA and
         sha(ci.git_blob(product, str(ci.GUARD.relative_to(ROOT)))) == ci.GUARD_SHA,
         'workspace guard is not pinned in the current product commit')
    need(ex.get('guard_limits') == {'free_bytes': 5 * 1024**3, 'target_bytes': 1024**3},
         'fresh workspace guard limits differ from reviewed limits')
    runner = ex.get('runner', {})
    need(runner.get('path') == str(WORKSPACE_RUNNER) and runner.get('sha256') == WORKSPACE_RUNNER_SHA and
         sha_file(WORKSPACE_RUNNER) == WORKSPACE_RUNNER_SHA,
         'fresh workspace runner path or actual SHA mismatch')
    source_runner = runner.get('derived_from')
    need(isinstance(source_runner, str) and source_runner.startswith('openspec/'),
         'workspace runner lacks its repository source provenance')
    source_runner_path = ROOT / source_runner
    need(source_runner_path.is_file() and
         sha(ci.git_blob(product, source_runner)) == sha_file(source_runner_path),
         'workspace runner source provenance differs from product Git/live bytes')

    pins_before = ex.get('source_pins_before_format')
    pins_after = ex.get('source_pins')
    need(isinstance(pins_before, dict) and pins_before == pins_after,
         'fresh workspace before/after source pins differ')
    need(len(pins_before) >= MIN_SOURCE_PINS, 'fresh workspace source-pin inventory is unexpectedly incomplete')
    need('tests/fixtures/corpus-fingerprint.json' in pins_before,
         'fresh workspace pins omit its runtime-read corpus fingerprint')
    product_pin_hashes = ci.blob_sha256s(product, pins_before)
    need(product_pin_hashes == pins_before, 'fresh workspace source inputs differ from product commit Git blobs')
    live_pin_hashes = {path: sha_file(ROOT / path) for path in pins_before}
    need(live_pin_hashes == pins_before, 'fresh workspace source inputs differ from live tree')

    by_id, by_named, inventory = ci.metadata_targets(metadata)
    declared = [tuple(item) for item in ex.get('target_inventory', [])]
    need(len(declared) == len(set(declared)) and set(declared) == set(inventory),
         'fresh workspace target inventory differs from metadata')
    need(len(inventory) == TARGET_COUNT, 'fresh workspace did not enumerate exactly 359 Cargo targets')
    need(ex.get('format_command', {}).get('argv') == ['cargo', 'fmt', '--all', '--', '--check'] and
         ex['format_command'].get('exit_code') == 0 and ex['format_command'].get('guard_stop') is None,
         'fresh workspace format gate did not pass')
    need(ex.get('metadata_command', {}).get('argv') ==
         ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'] and
         ex['metadata_command'].get('exit_code') == 0 and ex['metadata_command'].get('guard_stop') is None,
         'fresh workspace Cargo metadata command did not pass')
    for row in (ex['format_command'], ex['metadata_command']):
        need(row.get('peak_target_bytes', 0) <= 1024**3 and row.get('free_bytes_after', 0) >= 5 * 1024**3,
             'fresh workspace preflight command violated resource limits')
    ci.verified_stream(ex['format_command']['streams']['stdout'])
    ci.verified_stream(ex['format_command']['streams']['stderr'])
    md_stdout = ci.verified_stream(ex['metadata_command']['streams']['stdout'])
    need(md_stdout == metadata_raw, 'saved metadata differs from fresh workspace metadata raw output')
    commands = ex.get('commands', [])
    need(ex.get('expected_batch_count') == 95 and len(commands) == 95 and
         ex.get('expected_batch_count') == len(commands) and
         [row.get('index') for row in commands] == list(range(2, 2 + len(commands))),
         'fresh workspace must contain the complete 95-batch run in order')
    all_observed = []
    for row in commands:
        need(row.get('exit_code') == 0 and row.get('guard_stop') is None,
             f"fresh workspace batch {row.get('index')} failed or hit a guard")
        need(row.get('peak_target_bytes', 0) <= 1024**3 and row.get('free_bytes_after', 0) >= 5 * 1024**3,
             f"fresh workspace batch {row.get('index')} violated resource limits")
        observed = ci.local_command_observation(row, by_id, by_named)
        ci.check_workspace_argv(row.get('argv', []), observed, by_named)
        all_observed.extend(observed)
    observed_keys = [item['key'] for item in all_observed]
    need(len(observed_keys) == len(set(observed_keys)) and set(observed_keys) == set(inventory),
         'fresh workspace did not execute every Cargo target exactly once')
    need(ex.get('workspace_summary_count') == len(all_observed) == TARGET_COUNT,
         'fresh workspace summary count differs from its full target inventory')
    totals = [sum(item['counts'][i] for item in all_observed) for i in range(3)]
    need(ex.get('workspace_totals') == totals, 'fresh workspace totals differ from reconstructed raw outcomes')
    local = {'execution': str(execution_path), 'execution_sha256': ex_sha,
             'metadata': str(metadata_path), 'metadata_sha256': metadata_sha,
             'seed': ex['seed'], 'target_count': len(inventory), 'batch_count': len(commands),
             'workspace_totals': totals, 'source_pin_count': len(pins_before),
             'target_observations': all_observed}
    return ex, metadata, local


# One count is intentionally set from the pinned v9 adapter source.
CI_SCAN_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'


def verify_product_build(product: str, workspace_pins: dict[str, str]):
    need(sha_file(BUILD_EVIDENCE) == BUILD_EVIDENCE_SHA, 'frozen arithmetic-stack v2 build-evidence helper pin mismatch')
    need(sha_file(BUILD_RUNNER) == BUILD_RUNNER_SHA, 'frozen arithmetic-stack v2 CLI build runner pin mismatch')
    build_raw = BUILD_RECORD.read_bytes()
    build = json.loads(build_raw)
    cli_pin = build.get('cli')
    need(isinstance(cli_pin, dict), 'candidate build has no CLI pin')
    result = verify_build(BUILD_RECORD, cli_pin, cli_pin.get('sha256'))
    product_pins = build.get('product_pins_before')
    need(isinstance(product_pins, dict) and product_pins == build.get('product_pins_after'),
         'candidate CLI production source pins changed during build')
    dynamic_paths = set(result.get('dynamic_product_paths', []))
    need(dynamic_paths and set(product_pins) == dynamic_paths and
         result.get('product_source_pin_count') == len(dynamic_paths),
         'candidate CLI production source pins differ from the current v2 runner dynamic selection')
    need(set(product_pins) <= set(workspace_pins) and
         all(workspace_pins[path] == digest for path, digest in product_pins.items()),
         'CLI production source pins are not contained in the full workspace pins')
    git = ci.blob_sha256s(product, product_pins)
    need(git == product_pins, 'CLI production source pins differ from current product Git blobs')
    live = {path: sha_file(ROOT / path) for path in product_pins}
    need(live == product_pins, 'CLI production source pins differ from live product bytes')
    return {'build_path': str(BUILD_RECORD), 'build_sha256': sha(build_raw),
            'cli': cli_pin, 'product_source_pin_count': result['product_source_pin_count'],
            'production_source_pins_git_bound': len(git), 'build_evidence_helper_sha256': BUILD_EVIDENCE_SHA,
            'build_runner_sha256': BUILD_RUNNER_SHA}


def verify_checklist_sources(product: str, local_pins: dict[str, str]):
    expected = checklist_pins()
    need(set(expected) <= set(local_pins), 'fresh workspace pins omit a checklist-pinned product input')
    git = ci.blob_sha256s(product, expected)
    need(git == expected, 'checklist source pins differ from current product Git blobs')
    live = {path: sha_file(ROOT / path) for path in expected}
    need(live == expected and live == {path: local_pins[path] for path in expected},
         'checklist source pins differ from full-workspace live pins')
    return expected


NEW_REQUIRED = {
    'crates/jarde-jvm/src/canonical.rs': (
        'shared_conditional_subroutine_is_covered_per_call_path',
        'clone_coverage_rejects_a_disconnected_internal_node',
        'clone_coverage_rejects_a_disconnected_self_cycle',
        'clone_coverage_rejects_wrong_normal_call_return_and_exception_paths',
        'clone_coverage_accepts_a_nested_return_continuation',
        'an_uncovered_nested_return_cannot_cover_its_outer_continuation',
        'explicit_dead_callsite_seed_stays_unreachable_after_coverage',
        'clone_coverage_budgets_storage_work_and_assembly_cancellation',
        'a_clone_exception_edge_reaches_its_handler_and_ret_restores_the_caller_path',
    ),
    'crates/jarde-java/src/region.rs': (
        'real_compound_statement_with_two_payload_arms_has_one_shared_join',
        'real_compound_non_tree_conditions_refuse_without_walker_state_leaks',
        'real_legacy_clone_target_with_different_path_refuses_switch_certificate',
        'real_nested_switch_target_refuses_candidate_certificate_without_partial_result',
    ),
    'crates/jarde-java/src/build.rs': (
        'compound_builder_proof_and_emission_stops_restore_the_trial',
    ),
    'tests/p3_switch_forward_join.rs': (
        'cross_case_goto_target_owns_the_shared_forward_continuation',
        'looping_arm_backedge_does_not_gain_a_forward_join',
    ),
    'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs': (
        'cf12_conditional_switch_keeps_exact_case_and_exit_sources',
        'cf12_ft2_outer_if_owns_switch_join_and_tail_with_physical_origins',
        'cf12_recovery_with_pre_cancelled_public_budget_publishes_no_partial_artifact',
        'cf12_switch_proof_stops_at_observed_analysis_step_boundaries_without_an_artifact',
        'compound_statement_conditions_have_exact_physical_origins',
        'compound_float_condition_preserves_nan_sensitive_comparison_origins',
        'compound_whole_recovery_late_analysis_step_stop_publishes_no_partial_artifact',
        'compound_pre_cancelled_public_recovery_publishes_no_partial_artifact',
    ),
    'tests/p3_reference_slot_lifetimes.rs': (RENAMED_NULL_TEST,),
    'tests/p3_immediate_functional_receivers.rs': (
        'an_overdeep_lambda_helper_renames_instead_of_walking_an_unbounded_body',
    ),
}


def source_test_names(product: str, pins: dict[str, str]):
    result = {}
    for source, names in NEW_REQUIRED.items():
        need(source in pins, 'fresh workspace source pins omit required regression source: ' + source)
        raw = ci.git_blob(product, source)
        need(sha(raw) == pins[source] == sha_file(ROOT / source),
             'required test source is not bound to product Git/live pins: ' + source)
        text = raw.decode('utf-8')
        for name in names:
            pattern = r'(?m)^\s*(?:pub\s+)?fn\s+' + re.escape(name) + r'\s*\('
            need(re.search(pattern, text) is not None,
                 f'required test function is absent from pinned source {source}: {name}')
        result[source] = {'sha256': sha(raw), 'test_names': list(names)}
    return result


def metadata_target_for_source(source: str, by_named: dict):
    """Resolve a Rust source to its Cargo target using fresh metadata, not package-name guesses."""
    exact = [row for row in by_named.values() if row['source_path'] == source]
    if not exact:
        absolute = (ROOT / source).resolve()
        candidates = [row for row in by_named.values()
                      if row['kind'] == 'lib' and absolute.is_relative_to(Path(row['package_dir']).resolve())]
        need(candidates, 'source is outside all Cargo library package directories: ' + source)
        deepest = max(len(Path(row['package_dir']).resolve().parts) for row in candidates)
        exact = [row for row in candidates if len(Path(row['package_dir']).resolve().parts) == deepest]
    need(len(exact) == 1, 'source does not map to exactly one Cargo target in current metadata: ' + source)
    return exact[0]


def target_key(row: dict):
    return (row['package'], row['name'], row['kind'])


def verify_named_tests(local_observations, product: str, pins: dict[str, str], metadata: dict):
    # Retain the earlier accepted conditional control tests as a subset, then require this increment.
    legacy = ci.verify_required_tests(local_observations)
    source_details = source_test_names(product, pins)
    _, by_named, _ = ci.metadata_targets(metadata)
    targets = {row['key']: row for row in local_observations}
    source_targets = {}
    checked = []
    for source, names in NEW_REQUIRED.items():
        metadata_row = metadata_target_for_source(source, by_named)
        key = target_key(metadata_row)
        source_targets[source] = {'target': key, 'target_source_path': metadata_row['source_path']}
        need(key in targets, 'mapped Cargo target was not run in the fresh workspace: ' + repr(key))
        local = targets[key]
        outcomes = local['outcomes']
        for name in names:
            matches = [(actual, state) for actual, state in outcomes
                       if actual == name or actual.endswith('::' + name)]
            need(len(matches) == 1 and matches[0][1] == 'ok',
                 f'required current-product test did not pass in fresh local target: {source}::{name}')
            if source in ('crates/jarde-jvm/src/canonical.rs', 'crates/jarde-java/src/region.rs',
                          'crates/jarde-java/src/build.rs'):
                need('::tests::' in matches[0][0], 'unit test outcome lacks its test module identity: ' + name)
        checked.append({'source_path': source, 'target': key, 'passed_test_names': list(names)})

    expected_counts_by_source = {
        'crates/jarde-jvm/src/lib.rs': (243, 0, 0),
        'crates/jarde-java/src/lib.rs': (351, 0, 0),
        'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs': (8, 0, 0),
    }
    summary_checks = {}
    for source, expected in expected_counts_by_source.items():
        key = target_key(metadata_target_for_source(source, by_named))
        need(key in targets and targets[key]['counts'] == expected,
             f'fresh local target {key!r} does not have the requested exact current summary')
        summary_checks[str(key)] = targets[key]['counts']
    null_key = target_key(metadata_target_for_source('tests/p3_reference_slot_lifetimes.rs', by_named))
    need(null_key in targets, 'fresh local NullThenBuilder integration target is absent')
    null_outcomes = dict(targets[null_key]['outcomes'])
    need(null_outcomes.get(RENAMED_NULL_TEST) == 'ok' and MASKED_OLD_NULL_TEST not in null_outcomes,
         'fresh local NullThenBuilder target lacks renamed passing test or retains stale test name')
    return {'prior_controls': legacy, 'increment_tests': checked,
            'source_text_checks': source_details, 'source_to_metadata_target': source_targets,
            'target_summary_checks': summary_checks,
            'null_test_rename': {'target': null_key, 'passed': RENAMED_NULL_TEST,
                                 'stale_name_absent': MASKED_OLD_NULL_TEST not in null_outcomes}}


def bind_public(raw_name: str, status: str, local_target: dict):
    if '***' not in raw_name:
        return raw_name
    pattern = '^' + '[^\\n]*'.join(re.escape(part) for part in raw_name.split('***')) + '$'
    candidates = [name for name, state in local_target['outcomes']
                  if state == status and re.fullmatch(pattern, name)]
    need(len(candidates) == 1,
         'masked CI public label does not uniquely bind to a same-status local outcome: ' +
         repr((local_target['key'], raw_name, status)))
    return candidates[0]


def parse_ci_group(group: str, seed: str, local_targets: list[dict], redactions: list[dict]):
    need(seed in group, 'CI workspace log group omits its expected PROPTEST_RNG_SEED')
    headers = []
    for line in group.splitlines():
        if 'Running ' in ci.clean(line):
            target = ci.parse_ci_header(line, local_targets)
            if target is not None:
                headers.append(target)
    blocks = ci.stdout_blocks_cargo_json(group.encode())
    need(len(headers) == len(local_targets) == len(blocks),
         'CI group does not contain the complete 359 target header/result inventory')
    keys = [target['key'] for target in headers]
    need(len(keys) == len(set(keys)) and set(keys) == {target['key'] for target in local_targets},
         'CI target headers omit, duplicate, or introduce a local target')
    local_by_key = {target['key']: target for target in local_targets}
    outcomes = []
    for header, block in zip(headers, blocks):
        local = local_by_key[header['key']]
        bound = []
        for raw_name, status in block['outcomes']:
            full_name = bind_public(raw_name, status, local)
            bound.append((full_name, status))
            if full_name != raw_name:
                redactions.append({'seed': seed, 'target': list(local['key']),
                                   'raw_public_label': raw_name, 'bound_local_test_name': full_name,
                                   'status': status, 'same_status_unique_candidate': True})
        need(len({name for name, _ in bound}) == len(bound),
             'masked CI label binding created duplicate test names')
        need(block['counts'] == local['counts'] and tuple(bound) == local['outcomes'],
             'CI per-target named outcomes/summary differ from fresh local workspace: ' + repr(local['key']))
        outcomes.append({'package': local['key'][0], 'name': local['key'][1], 'kind': local['key'][2],
                         'source_path': local['source_path'], 'counts': list(block['counts']),
                         'test_outcomes': [{'name': name, 'status': state} for name, state in bound]})
    totals = [sum(target['counts'][i] for target in outcomes) for i in range(3)]
    return {'seed': seed, 'target_count': len(outcomes), 'named_outcome_count': sum(len(t['test_outcomes']) for t in outcomes),
            'workspace_totals': totals, 'targets': outcomes}


def verify_ci_capture(capture: Path, product: str, run_id: str, local_targets: list[dict]):
    summary, steps = ci.verify_capture(capture, product, run_id)
    workflow_raw = ci.git_blob(product, '.github/workflows/ci.yml')
    need(sha(workflow_raw) == WORKFLOW_SHA and sha_file(ROOT / '.github/workflows/ci.yml') == WORKFLOW_SHA,
         'current product workflow differs from checklist pin')
    workflow_seeds = ci.CI_SEEDS_RE.findall(workflow_raw.decode('utf-8'))
    need(tuple(workflow_seeds) == SEEDS and workflow_raw.decode('utf-8').count('run: ' + WORKSPACE_CMD) == 2,
         'current product workflow does not define the two reviewed full-workspace seeds')
    raw = gzip.decompress((capture / 'ci-stable-job-v2.log.gz').read_bytes())
    lines = ci.strip_log(raw).splitlines()
    groups = ci.command_groups(lines, WORKSPACE_CMD)
    need(len(groups) == 2, 'stable CI raw log does not contain exactly two full-workspace runs')
    redactions = []
    seed_runs = [parse_ci_group(group, seed, local_targets, redactions)
                 for group, seed in zip(groups, workflow_seeds)]
    need([row['seed'] for row in seed_runs] == list(SEEDS), 'CI workspace seed order mismatch')
    need(seed_runs[0]['target_count'] == TARGET_COUNT == seed_runs[1]['target_count'],
         'CI seed logs do not each contain the full 359-target inventory')
    need(seed_runs[0]['workspace_totals'] == seed_runs[1]['workspace_totals'],
         'CI seed workspace totals differ')
    return {'summary': summary, 'steps': steps, 'seed_runs': seed_runs,
            'masked_public_label_mappings': redactions,
            'public_label_rule': {'path': str(PUBLIC_LABEL_RULE), 'sha256': PUBLIC_LABEL_RULE_SHA,
                                  'rule': 'A *** public label binds only to exactly one same-status outcome in the corresponding product-pinned local target; captured raw logs remain unchanged.'}}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--product-commit', required=True)
    ap.add_argument('--run-id', required=True)
    ap.add_argument('--capture', type=Path, required=True)
    ap.add_argument('--workspace', type=Path, required=True,
                    help='directory containing the completed fresh full-workspace execution.json and metadata.json')
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    need(re.fullmatch(r'[0-9a-f]{40}', args.product_commit) is not None,
         'product commit must be lowercase 40-hex')
    need(args.run_id.isdecimal(), 'run ID must be decimal')
    out = args.out.resolve()
    need(not out.exists(), 'refusing to overwrite acceptance output: ' + str(out))
    for path, expected in ((HELPER_PATH, HELPER_SHA), (CAPTURE_PATH, CAPTURE_SHA),
                           (PUBLIC_LABEL_RULE, PUBLIC_LABEL_RULE_SHA), (CHECKLIST, CHECKLIST_SHA),
                           (BUILD_EVIDENCE, BUILD_EVIDENCE_SHA), (BUILD_RUNNER, BUILD_RUNNER_SHA),
                           (WORKSPACE_RUNNER, WORKSPACE_RUNNER_SHA)):
        need(path.is_file() and sha_file(path) == expected, 'reviewed helper pin mismatch: ' + str(path))

    ex, metadata, local = verify_workspace(args.product_commit, args.workspace)
    checklist = verify_checklist_sources(args.product_commit, ex['source_pins'])
    candidate = verify_product_build(args.product_commit, ex['source_pins'])
    required = verify_named_tests(local['target_observations'], args.product_commit, ex['source_pins'], metadata)
    ci_result = verify_ci_capture(args.capture.resolve(), args.product_commit, args.run_id,
                                  local['target_observations'])
    acceptance = {
        'schema': 'arithmetic-stack-own-ci-current-product-acceptance-luna-v1',
        'status': 'accepted',
        'product_commit': args.product_commit,
        'run_id': int(args.run_id),
        'ci': {'jobs': 4, 'steps': ci_result['steps'], 'seed_runs': ci_result['seed_runs'],
               'run_summary': ci_result['summary'],
               'aggregate_counts_predeclared': False,
               'masked_public_label_mappings': ci_result['masked_public_label_mappings'],
               'public_label_rule': ci_result['public_label_rule']},
        'fresh_local_workspace': {k: v for k, v in local.items() if k != 'target_observations'},
        'workspace_runner_contract': {'path': str(WORKSPACE_RUNNER), 'sha256': WORKSPACE_RUNNER_SHA,
                                      'complete_batch_count': 95, 'target_count': TARGET_COUNT},
        'candidate_cli_build': candidate,
        'required_tests': required,
        'checklist_source_pin_count': len(checklist),
        'workspace_target_count': TARGET_COUNT,
        'evidence': {'capture': str(args.capture.resolve()),
                     'capture_execution_sha256': sha_file(args.capture.resolve() / 'capture-execution-root-v2.json'),
                     'capture_collector_sha256': CAPTURE_SHA,
                     'workspace_execution': local['execution'],
                     'workspace_execution_sha256': local['execution_sha256'],
                     'workspace_metadata': local['metadata'],
                     'workspace_metadata_sha256': local['metadata_sha256'],
                     'workspace_runner_path': str(WORKSPACE_RUNNER),
                     'workspace_runner_sha256': WORKSPACE_RUNNER_SHA,
                     'build_evidence_helper_sha256': BUILD_EVIDENCE_SHA},
        'acceptance_scope': 'Exact current product Git commit and completed 4-job/52-step CI run; both workflow workspace seeds reproduce all 359 full target summaries and named outcomes from the fresh no-carry 95-batch local workspace run, with same-status unique binding for masked public labels; complete local and CLI production input pins are bound to product Git blobs and live bytes.'
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('x', encoding='utf-8') as stream:
        stream.write(json.dumps(acceptance, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': 'accepted', 'product_commit': args.product_commit,
                      'run_id': int(args.run_id), 'jobs': 4, 'steps': ci_result['steps'],
                      'target_count': TARGET_COUNT,
                      'workspace_totals': [row['workspace_totals'] for row in ci_result['seed_runs']],
                      'masked_public_labels': len(ci_result['masked_public_label_mappings'])}))


if __name__ == '__main__':
    try:
        main()
    except Exception as exc:
        print('current-product own-CI verification failed: ' + str(exc), file=sys.stderr)
        raise
