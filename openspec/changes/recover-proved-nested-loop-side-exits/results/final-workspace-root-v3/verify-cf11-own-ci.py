#!/usr/bin/env python3
"""Bind current-product GitHub CI to the fresh complete compound workspace replay."""
from __future__ import annotations
import argparse
import gzip
import hashlib
import importlib.util
import json
import re
import stat
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
COMMITTED_BASE = ROOT / 'openspec/changes/bound-arithmetic-render-stack-frames/results/own-product-ci-root-v1/private-verifier/verify-own-ci.py'
COMMITTED_BASE_SHA = 'bacc079842691181b3e6df3625fcd8a531cf61af4188c6f2a45cc7c569c0a014'
HELPER_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-ci-root-v5.py'
HELPER_SHA = 'bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca'
CAPTURE_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py'
CAPTURE_SHA = 'ed889bf6af08247efcce3c889cd1c790d6829d0ea12e9c0d053bdef801994be3'
PUBLIC_LABEL_RULE = Path('/private/tmp/jarde-conditional-ci-public-labels-root-v1.py')
PUBLIC_LABEL_RULE_SHA = 'd7a6bcebb703b1520332cf4c7419c50d21d919d6a0c6123162d0d4d78bbcac65'
WORKSPACE_RUNNER = Path('/private/tmp/jarde-compound-workspace-batches-root-v1.py')
WORKSPACE_RUNNER_SHA = '2ac64ccdf7ed6c8f90f37b8ae34693eb16206cba493ca4a3e261de993a08235b'
WORKSPACE_SCHEMA = 'compound-full-workspace-batched-root-v1'
SEEDS = ('5350648285461741569', '5350648285461741570')
WORKSPACE_CMD = 'cargo test --workspace --all-targets --all-features --locked'
WORKFLOW_SHA = 'b545890a55cc4fda31d70b332889c111d789afbf0949581e96bf2b94ea6cd18a'
MASKED_OLD_NULL_TEST = 'null_first_exact_builder_writes_refine_only_the_local_type'
RENAMED_NULL_TEST = 'null_first_exact_builder_writes_refine_local_type_and_compound_condition'

spec = importlib.util.spec_from_file_location('conditional_ci_v5_helpers', HELPER_PATH)
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)


def need(ok: bool, message: str):
    if not ok:
        raise ValueError(message)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(Path(path).read_bytes())


def load_json(path: Path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def workspace_input_paths():
    paths={ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'.github/workflows/ci.yml',ROOT/'tests/fixtures/corpus-fingerprint.json'}
    for base in ('src','crates','tests'):
        paths.update((ROOT/base).rglob('*.rs'))
    paths.update((ROOT/'crates').glob('*/Cargo.toml'))
    for asset_root in (ROOT/'tests/fixtures',ROOT/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):
        paths.update(p for p in asset_root.rglob('*') if p.is_file())
    for path in list(paths):
        if path.suffix=='.rs':
            for rel in re.findall(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"',path.read_text(encoding='utf-8')):
                child=(path.parent/rel).resolve()
                need(child.is_relative_to(ROOT.resolve()),'literal include escapes workspace: '+str(child))
                if child.is_file(): paths.add(child)
    return {p.relative_to(ROOT).as_posix() for p in paths}

def verify_workspace(product: str, workspace: Path):
    workspace = workspace.resolve(strict=True)
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
    expected_paths=workspace_input_paths()
    need(set(pins_before)==expected_paths, 'fresh workspace pins differ from the runner-derived source and include/fixture closure')
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
    target_count=len(inventory)
    need(target_count>0, 'fresh workspace metadata has no workspace targets')
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
    need(isinstance(ex.get('expected_batch_count'), int) and ex['expected_batch_count'] > 0 and
         len(commands) == ex['expected_batch_count'] and
         [row.get('index') for row in commands] == list(range(2, 2 + len(commands))),
         'fresh workspace batch inventory is incomplete or out of order')
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
    need(ex.get('workspace_summary_count') == len(all_observed) == target_count,
         'fresh workspace summary count differs from its full target inventory')
    totals = [sum(item['counts'][i] for item in all_observed) for i in range(3)]
    need(ex.get('workspace_totals') == totals, 'fresh workspace totals differ from reconstructed raw outcomes')
    local = {'execution': str(execution_path), 'execution_sha256': ex_sha,
             'metadata': str(metadata_path), 'metadata_sha256': metadata_sha,
             'seed': ex['seed'], 'target_count': target_count, 'batch_count': len(commands),
             'workspace_totals': totals, 'source_pin_count': len(pins_before),
             'fixture_and_literal_input_count': len(expected_paths - {'Cargo.toml','Cargo.lock','.github/workflows/ci.yml','tests/fixtures/corpus-fingerprint.json'}),
             'target_observations': all_observed}
    return ex, metadata, local


# Scanner adapter identity remains pinned to the accepted workspace runner.
CI_SCAN_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'


def verify_final_cli_build(product: str, workspace_pins: dict[str, str], build_record: Path,
                           runner_path: Path, runner_sha: str,
                           expected_cli_path: Path, expected_cli_sha: str):
    need(re.fullmatch(r'[0-9a-f]{64}', runner_sha) is not None, 'CLI build runner SHA must be lowercase SHA-256')
    need(sha_file(runner_path) == runner_sha, 'CLI build runner file SHA mismatch')
    raw = build_record.read_bytes(); build = json.loads(raw)
    need(build.get('schema') == 'cf11-side-exits-cli-build-root-v10' and
         build.get('status') == 'passed_candidate_build_only',
         'CF11 CLI candidate build receipt is not a successful v10 build')
    need(build.get('runner_sha256_before') == runner_sha == build.get('runner_sha256_after'),
         'CLI build runner changed during capture or does not match supplied runner pin')
    need(build.get('source_pins_before') == workspace_pins == build.get('source_pins_after_format') == build.get('source_pins_after'),
         'CLI build source pins differ from fresh workspace Git/live closure')
    need(build.get('target_absent') is True and not (ROOT / 'target').exists(),
         'CLI build receipt did not leave target absent after cargo clean')
    env_expected = {'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0',
      'CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}
    need(build.get('actual_env_overrides') == env_expected, 'CLI build environment overrides differ from reviewed guard settings')
    need(build.get('rust_min_stack') is None, 'CLI build unexpectedly overrides the default Rust thread stack')
    prior = build.get('prior_same_source_focused_receipt')
    need(isinstance(prior, dict) and isinstance(prior.get('path'), str) and
         re.fullmatch(r'[0-9a-f]{64}', prior.get('sha256', '')) is not None,
         'same-source focused receipt path/SHA pin is missing')
    prior_path = Path(prior['path'])
    prior_raw = prior_path.read_bytes()
    need(sha(prior_raw) == prior['sha256'], 'same-source focused receipt SHA mismatch')
    focused = json.loads(prior_raw)
    need(focused.get('schema') == 'cf11-side-exits-focused-root-v7' and
         focused.get('rust_min_stack') is None,
         'focused receipt schema or default-stack setting mismatch')
    old_pins = focused.get('source_pins_after')
    differences = build.get('focused_source_differences')
    reader_path = 'crates/jarde-reader/src/classfile.rs'
    region_path = 'crates/jarde-java/src/region.rs'
    expected_paths = {reader_path, region_path}
    need(isinstance(old_pins, dict) and set(old_pins) == set(workspace_pins) and
         isinstance(differences, list) and len(differences) == 2 and
         all(isinstance(row, dict) for row in differences),
         'v10 focused-source difference rows or prior pin inventory are malformed')
    rows = {row.get('path'): row for row in differences}
    need(set(rows) == expected_paths and len(rows) == len(differences),
         'v10 must record exactly the reader census and region predicate paths')
    for path in expected_paths:
        row = rows[path]
        need(row.get('previous_sha256') == old_pins.get(path) and
             row.get('current_sha256') == workspace_pins.get(path),
             'v10 focused-source row SHA does not match prior and current full pins: ' + path)
    changed = {name for name in workspace_pins if workspace_pins[name] != old_pins[name]}
    need(changed == expected_paths,
         'focused receipt differs from the full closure outside the two exact v10 source changes')
    current_reader = (ROOT / reader_path).read_text(encoding='utf-8')
    changed_text = ('// CF11 fixed/controls: two classes, five Code bodies, no handlers, thirty\n'
                    '            // branch targets (10 + 20), no jsr; independently counted from accepted javap.\n'
                    '            (1095, 4866, 468, 2912, 10)')
    need(current_reader.count(changed_text) == 1,
         'current reader census edit does not match the v10 runner inverse')
    restored_reader = current_reader.replace(changed_text, '(1093, 4861, 468, 2882, 10)', 1)
    need(sha(restored_reader.encode()) == old_pins[reader_path],
         'inverse census edit does not restore the exact focused classfile.rs bytes')
    current_region = (ROOT / region_path).read_text(encoding='utf-8')
    changed_predicate = '|| prefix.is_empty()'
    need(current_region.count(changed_predicate) == 1,
         'current region predicate edit does not match the v10 runner inverse')
    restored_region = current_region.replace(changed_predicate, '|| prefix.first().is_none()', 1)
    new_frame = ('        let mut frame = Frame {\n'
                 '            scope: Some((0..view.len()).collect()),\n'
                 '            ..Frame::default()\n'
                 '        };')
    old_frame = ('        let mut frame = Frame::default();\n'
                 '        frame.scope = Some((0..view.len()).collect());')
    need(restored_region.count(new_frame) == 1,
         'current Frame initializer edit does not match the v10 runner inverse')
    restored_region = restored_region.replace(new_frame, old_frame, 1)
    need(sha(restored_region.encode()) == old_pins[region_path],
         'inverse predicate and Frame initializer edits do not restore exact focused region.rs bytes')
    focus_commands = focused.get('commands', [])
    need(len(focus_commands) == 9 and
         all(row.get('index') == i and row.get('exit_code') == 0 and row.get('guard_stop') is None
             for i, row in enumerate(focus_commands[:7])),
         'focused receipt must retain its seven successful pre-build commands')
    wrong_bin = focus_commands[7]
    need(wrong_bin.get('index') == 7 and wrong_bin.get('argv') ==
         ['cargo','build','-p','jarde-cli','--bin','jarde','--locked'] and
         wrong_bin.get('exit_code') == 101 and wrong_bin.get('guard_stop') is None,
         'focused receipt must preserve the observed wrong-bin build failure')
    clean = focus_commands[8]
    need(clean.get('index') == 8 and clean.get('argv') == ['cargo','clean'] and
         clean.get('exit_code') == 0 and clean.get('guard_stop') is None,
         'focused failed-build receipt did not finish with cargo clean')
    commands = build.get('commands', [])
    expected = [['cargo','fmt','--all','--','--check'],
      ['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'],['cargo','clean']]
    need(len(commands) == len(expected), 'CLI build receipt must contain fmt, exact CLI build, and clean commands')
    for index, (row, argv) in enumerate(zip(commands, expected)):
        need(row.get('index') == index and row.get('argv') == argv and row.get('cwd') == str(ROOT) and
             row.get('exit_code') == 0 and row.get('guard_stop') is None, f'CLI build command {index} argv/result mismatch')
        need(row.get('peak_target_bytes', 2**63) <= 1024**3 and row.get('free_bytes_after', 0) >= 5*1024**3 and
             row.get('env_overrides') == env_expected, f'CLI build command {index} violated resource/environment guard')
        for name in ('stdout','stderr'):
            ref = row.get('streams', {}).get(name); need(isinstance(ref, dict), f'CLI build command {index} missing {name} pin')
            stream = Path(ref['path']).read_bytes()
            need(len(stream) == ref.get('bytes') and sha(stream) == ref.get('sha256'), f'CLI build command {index} {name} pin mismatch')
    need(expected_cli_path.resolve() == Path('/private/tmp/jarde-cf11-side-exits-candidate-cli-root-v10').resolve(),
         'candidate CLI path differs from the v10 frozen destination')
    cli = build.get('candidate_cli')
    need(isinstance(cli, dict) and cli.get('path') == str(expected_cli_path) and cli.get('sha256') == expected_cli_sha and
         cli.get('mode') == '0o555', 'candidate CLI path/SHA/mode differs from explicit root observation')
    cli_file = Path(cli['path'])
    need(cli_file.is_file() and cli_file.stat().st_size == cli.get('bytes') and sha_file(cli_file) == expected_cli_sha and
         stat.S_IMODE(cli_file.stat().st_mode) == 0o555, 'frozen candidate CLI bytes/hash/mode mismatch')
    need(isinstance(build.get('prior_same_source_focused_receipt'), dict) and
         build['prior_same_source_focused_receipt'].get('tests_passed') is True,
         'source base or successful same-source focused receipt missing')
    need(ci.blob_sha256s(product, workspace_pins) == workspace_pins and
         {path:sha_file(ROOT/path) for path in workspace_pins} == workspace_pins,
         'CLI build full source closure differs from product Git blobs or live bytes')
    return {'build_path':str(build_record.resolve()), 'build_sha256':sha(raw), 'cli':cli,
      'product_commit':product, 'product_source_pin_count':len(workspace_pins),
      'product_source_pins_match_workspace':True, 'runner_path':str(runner_path.resolve()),
      'runner_sha256':runner_sha, 'commands':3, 'target_absent_after_clean':True}


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
        'shared_stack_initializer_has_two_real_dup_outputs_before_builder_refusal',
        'cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir',
        'real_cf11_side_exit_proof_uses_the_reader_graph_and_has_a_real_stop_boundary',
    ),
    'crates/jarde-java/src/build.rs': (
        'compound_builder_proof_and_emission_stops_restore_the_trial',
    ),
    'tests/p3_for_latch.rs': (
        'computed_init_return_leaf_and_latch_keep_one_physical_for_source',
        'direct_call_initializer_is_written_once_in_the_for_header',
        'computed_initializer_call_order_matches_physical_store_anchors',
        'computed_for_controls_keep_fallback_boundaries_and_evidence_parity',
        'shared_stack_initializer_refuses_a_for_header_with_complete_physical_sources',
    ),
    'crates/jarde-java/tests/p3_loop_exit_gateways.rs': (
        'cf07_return_arm_loop_latch_is_derived_from_complete_for',
    ),
    'crates/jarde-java/tests/p3_loop_terminal_return.rs': (
        'cf07_unproved_terminal_candidates_keep_source_and_quote',
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
    'tests/p3_nested_loop_side_exits.rs': (
        'upstream_nested_loops4_is_structured_with_complete_physical_sources',
        'controlled_returns_keep_full_class_maps_for_both_physical_methods',
        'a_late_public_budget_stop_and_precancellation_publish_no_partial_body',
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

    core_name='refused_root_without_child_keeps_same_class_integer_constant_names'
    core_key=target_key(metadata_target_for_source('src/facade.rs',by_named))
    need(core_key in targets,'new facade API test target is absent from the fresh workspace')
    core_matches=[(name,state) for name,state in targets[core_key]['outcomes']
                  if name==core_name or name.endswith('::'+core_name)]
    need(len(core_matches)==1 and core_matches[0][1]=='ok',
         'single newly added facade API test is not exactly one passing local outcome')
    summary_checks={str(key):row['counts'] for key,row in targets.items()
                    if key in {target_key(metadata_target_for_source(src,by_named))
                               for src in ('crates/jarde-jvm/src/lib.rs','crates/jarde-java/src/lib.rs',
                                           'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs','tests/p3_for_latch.rs','src/facade.rs')}}
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
         'CI group does not contain the complete current metadata target header/result inventory')
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
         'current product workflow differs from reviewed workflow pin')
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
    need(seed_runs[0]['target_count'] == len(local_targets) == seed_runs[1]['target_count'],
         'CI seed logs do not each contain the full current metadata target inventory')
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
    ap.add_argument('--final-cli-build', type=Path, required=True, help='completed guarded build/freeze/clean execution.json')
    ap.add_argument('--final-cli-runner', type=Path, required=True)
    ap.add_argument('--final-cli-runner-sha256', required=True)
    ap.add_argument('--candidate-cli', type=Path, required=True)
    ap.add_argument('--candidate-cli-sha256', required=True)
    args = ap.parse_args()
    need(re.fullmatch(r'[0-9a-f]{40}', args.product_commit) is not None,
         'product commit must be lowercase 40-hex')
    need(args.run_id.isdecimal(), 'run ID must be decimal')
    out = args.out.resolve()
    need(not out.exists(), 'refusing to overwrite acceptance output: ' + str(out))
    for path, expected in ((HELPER_PATH, HELPER_SHA), (CAPTURE_PATH, CAPTURE_SHA),
                           (PUBLIC_LABEL_RULE, PUBLIC_LABEL_RULE_SHA),
                           (WORKSPACE_RUNNER, WORKSPACE_RUNNER_SHA),
                           (COMMITTED_BASE, COMMITTED_BASE_SHA)):
        need(path.is_file() and sha_file(path) == expected, 'reviewed helper pin mismatch: ' + str(path))

    ex, metadata, local = verify_workspace(args.product_commit, args.workspace)
    candidate = verify_final_cli_build(args.product_commit, ex['source_pins'],args.final_cli_build.resolve(),
                                       args.final_cli_runner.resolve(),args.final_cli_runner_sha256,
                                       args.candidate_cli.resolve(),args.candidate_cli_sha256)
    required = verify_named_tests(local['target_observations'], args.product_commit, ex['source_pins'], metadata)
    ci_result = verify_ci_capture(args.capture.resolve(), args.product_commit, args.run_id,
                                  local['target_observations'])
    acceptance = {
        'schema': 'cf11-own-ci-current-product-acceptance-luna-v1',
        'status': 'accepted',
        'product_commit': args.product_commit,
        'run_id': int(args.run_id),
        'ci': {'jobs': 4, 'steps': ci_result['steps'], 'seed_runs': ci_result['seed_runs'],
               'run_summary': ci_result['summary'],
               'aggregate_counts_predeclared': False,
               'masked_public_label_mappings': ci_result['masked_public_label_mappings'],
               'public_label_rule': ci_result['public_label_rule']},
        'fresh_local_workspace': {k: v for k, v in local.items() if k != 'target_observations'},
        'workspace_runner_contract': {'path': ex['runner']['path'], 'sha256': ex['runner']['sha256'],
                                      'complete_batch_count': local['batch_count'], 'target_count': local['target_count']},
        'final_cli_build': candidate,
        'required_tests': required,
        'workspace_target_count': local['target_count'],
        'evidence': {'capture': str(args.capture.resolve()),
                     'capture_execution_sha256': sha_file(args.capture.resolve() / 'capture-execution-root-v2.json'),
                     'capture_collector_sha256': CAPTURE_SHA,
                     'workspace_execution': local['execution'],
                     'workspace_execution_sha256': local['execution_sha256'],
                     'workspace_metadata': local['metadata'],
                     'workspace_metadata_sha256': local['metadata_sha256'],
                     'workspace_runner_path': ex['runner']['path'],
                     'workspace_runner_sha256': ex['runner']['sha256'],
                     'final_cli_build_sha256': candidate['build_sha256'],
                     'final_cli_runner_sha256': args.final_cli_runner_sha256,
                     'committed_verifier_base': str(COMMITTED_BASE), 'committed_verifier_base_sha256': COMMITTED_BASE_SHA},
        'acceptance_scope': 'Exact parameterized CF11 product Git commit and CI run; both workflow seeds reproduce every metadata-derived target and named outcome from the no-carry local workspace record; masked labels bind only to unique same-status local outcomes; the full dynamic source/include/fixture closure and frozen v10 candidate CLI are checked against product Git blobs and live bytes.'
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('x', encoding='utf-8') as stream:
        stream.write(json.dumps(acceptance, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': 'accepted', 'product_commit': args.product_commit,
                      'run_id': int(args.run_id), 'jobs': 4, 'steps': ci_result['steps'],
                      'target_count': local['target_count'],
                      'workspace_totals': [row['workspace_totals'] for row in ci_result['seed_runs']],
                      'masked_public_labels': len(ci_result['masked_public_label_mappings'])}))


if __name__ == '__main__':
    try:
        main()
    except Exception as exc:
        print('current-product own-CI verification failed: ' + str(exc), file=sys.stderr)
        raise
