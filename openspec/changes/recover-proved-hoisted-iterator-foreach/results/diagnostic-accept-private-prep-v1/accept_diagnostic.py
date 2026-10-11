#!/usr/bin/env python3
"""Read-only acceptance and archival of the pinned hoisted-iterator diagnostic."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import stat
import sys

ROOT_DEFAULT = Path('/Users/lordcasser/workspace/projects/jarde')
EXECUTION_SHA256 = '40eee52fea7659b3e438229e4ce68f56a751f80c3bf88946be784a67d8e18236'
FINAL_BUILD_SHA256 = '90ecba1b52177d7dfae590679603e4624c4e56a42ae2b58d0616d7078c1f46e2'
FINAL_BUILD_SCHEMA = 'constructor-conditional-final-lint-build-root-v1'
FINAL_BUILD_STATUS = 'passed_candidate_build_only'
FINAL_BUILD_RUNNER_SHA256 = '7bf3a61aec009aaa629efad18fe8e177b25d0e3b828de135d52eba498ea5c2c3'
CLI_BYTES = 61286928
CLI_SHA256 = 'a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241'
CLI_MODE = 0o555
SEED = '5350648285461741569'
EXPECTED_TEST = 'facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic'
SOURCE_COUNT = 4727
CANDIDATE_FILES = {'crates/jarde-java/src/build.rs', 'src/facade.rs'}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
DOCELLIPSIS_EXCEPTIONS = {
    ('../tests/fixtures/…', 'crates/jarde-jvm/src/test_fixtures.rs'),
    ('../tests/fixtures/…', 'crates/jarde-reader/src/test_fixtures.rs'),
}


def sha_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    if not path.is_file() or path.is_symlink():
        raise AssertionError(f'expected regular non-symlink file: {path}')
    return sha_bytes(path.read_bytes())


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def read_json(path: Path) -> dict:
    require(path.is_file() and not path.is_symlink(), f'missing regular JSON file: {path}')
    return json.loads(path.read_text(encoding='utf-8'))


def enumerate_source_closure(root: Path) -> set[str]:
    """Independent reproduction of the final-build v1 in-tree source enumeration."""
    paths = {root / 'Cargo.toml', root / 'Cargo.lock', root / '.github/workflows/ci.yml',
             root / 'tests/fixtures/corpus-fingerprint.json'}
    for base in ('src', 'crates', 'tests'):
        paths.update((root / base).rglob('*.rs'))
    paths.update((root / 'crates').glob('*/Cargo.toml'))
    for base in ('tests/fixtures',
                 'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):
        paths.update(path for path in (root / base).rglob('*') if path.is_file())
    for path in list(paths):
        if path.suffix != '.rs':
            continue
        rel_source = str(path.relative_to(root))
        for rel_include in INCLUDE_RE.findall(path.read_text(encoding='utf-8')):
            included = (path.parent / rel_include).resolve()
            require(included.is_relative_to(root), f'include escapes workspace: {path}: {rel_include}')
            if not included.is_file():
                require((rel_include, rel_source) in DOCELLIPSIS_EXCEPTIONS,
                        f'missing source include: {path}: {rel_include}')
                continue
            paths.add(included)
    rels = set()
    for path in paths:
        require(path.is_file() and not path.is_symlink(), f'missing/symlink closure input: {path}')
        rels.add(str(path.relative_to(root)))
    return rels


def verify_stream(ref: dict, expected_path: Path | None = None) -> None:
    path = Path(ref['path'])
    require(path.is_file() and not path.is_symlink(), f'raw stream is not a regular file: {path}')
    if expected_path is not None:
        require(path.resolve() == expected_path.resolve(), f'raw stream path mismatch: {path}')
    raw = path.read_bytes()
    require(len(raw) == ref['bytes'], f'raw stream byte count mismatch: {path}')
    require(sha_bytes(raw) == ref['sha256'], f'raw stream SHA mismatch: {path}')


def verify_build_receipt(execution: dict, root: Path) -> tuple[dict, dict[str, str]]:
    receipt = execution['final_build_receipt']
    build_path = Path(receipt['path'])
    require(sha_file(build_path) == FINAL_BUILD_SHA256 == receipt['sha256'],
            'final-build receipt SHA mismatch')
    build = read_json(build_path)
    require(build['schema'] == FINAL_BUILD_SCHEMA and build['status'] == FINAL_BUILD_STATUS,
            'unexpected final-build receipt schema/status')
    require(receipt['schema'] == build['schema'] and receipt['status'] == build['status'],
            'diagnostic does not bind the actual final-build receipt fields')
    pins = build.get('source_pins_before')
    require(isinstance(pins, dict) and len(pins) == SOURCE_COUNT, 'final-build source pin count is not 4,727')
    require(build.get('source_pins_after') == pins, 'final-build before/after source pins differ')
    require(execution.get('source_pins_before') == pins, 'diagnostic source baseline differs from final build')
    require(execution.get('source_pins_after_restore') == pins,
            'diagnostic restored source pins differ from final build')
    require(execution.get('source_closure_count') == SOURCE_COUNT, 'diagnostic closure count mismatch')
    require(execution.get('source_enumeration_matches_final_build') is True
            and execution.get('source_enumeration_after_restore_matches') is True,
            'diagnostic independent enumeration did not match both times')
    require(execution.get('source_pins_match_before') is True, 'diagnostic source restore comparison failed')
    require(execution.get('restore') == {
        'attempted': True,
        'verified_files': {'crates/jarde-java/src/build.rs': True, 'src/facade.rs': True},
        'verified': True,
    }, 'diagnostic restore receipt is incomplete')
    require(execution.get('temporary_source_pins_only_two_candidates_changed') is True,
            'candidate source delta is not recorded as exactly temporary')
    candidates = execution.get('candidate_sources')
    require(isinstance(candidates, dict) and set(candidates) == CANDIDATE_FILES,
            'temporary candidate file set is not the exact two-file set')
    require(set(candidates).issubset(pins), 'candidate files are outside final-build source closure')
    prep = Path(execution['candidate_prep'])
    candidate_pins_path = Path(execution['candidate_pins']['path'])
    patch_path = Path(execution['candidate_patch']['path'])
    require(candidate_pins_path == prep / 'pins.json' and patch_path == prep / 'candidate.patch',
            'candidate artifacts are not rooted in the pinned private prep directory')
    require(sha_file(candidate_pins_path) == execution['candidate_pins']['sha256'],
            'candidate pins SHA mismatch')
    require(sha_file(patch_path) == execution['candidate_patch']['sha256'], 'candidate patch SHA mismatch')
    cp = read_json(candidate_pins_path)
    require(cp['schema'] == 'jarde-hoisted-iterator-same-ir-diagnostic-candidate-v1'
            and cp['status'] == 'private static candidate; unapplied, uncompiled, and unrun',
            'candidate pins do not describe the expected private candidate')
    pin_base = {row['path']: row['sha256'] for row in cp['base_source_pins']}
    pin_candidates = {row['path']: row['sha256'] for row in cp['candidate_files']}
    require(len(pin_candidates) == 2, 'candidate pin list is not exactly two files')
    for rel, row in candidates.items():
        base_path = root / rel
        cand_path = Path(row.get('path', str(prep / ('build.rs.candidate' if rel.endswith('build.rs') else 'facade.rs.candidate'))))
        require(row['base_sha256'] == pins[rel] == pin_base[str(base_path)], f'base pin mismatch: {rel}')
        require(row['candidate_sha256'] == pin_candidates[str(cand_path)], f'candidate pin mismatch: {rel}')
        require(sha_file(cand_path) == row['candidate_sha256'], f'candidate file SHA mismatch: {rel}')
    # Live source and frozen closure must agree byte-for-byte after restoration.
    require(enumerate_source_closure(root) == set(pins), 'independent live closure differs from 4,727-file receipt')
    for rel, digest in pins.items():
        require(sha_file(root / rel) == digest, f'live restored source mismatch: {rel}')
    require(execution.get('external_class', {}).get('separately_pinned_outside_source_closure') is True,
            'external frozen class was not separately pinned')
    external = execution['external_class']
    external_path = Path(external['path'])
    require(sha_file(external_path) == external['sha256'] == '8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231',
            'external frozen input SHA mismatch')
    require(external_path.stat().st_size == external['bytes'] and str(external_path) not in pins,
            'external class size/path closure boundary mismatch')

    build_runner = None
    for path_text, digest in execution['guard_tool_pins_before'].items():
        if path_text.endswith('jarde-constructor-conditional-final-lint-build-root-v1.py'):
            build_runner = Path(path_text)
            require(digest == FINAL_BUILD_RUNNER_SHA256, 'final-build runner SHA differs from fixed pin')
            require(sha_file(build_runner) == digest, 'final-build runner live SHA mismatch')
    require(build_runner is not None, 'diagnostic does not pin the final-build runner')
    require(execution['guard_tool_pins_after'] == execution['guard_tool_pins_before']
            and execution.get('guard_tool_pins_match_before') is True,
            'guard/final-build tool pins changed during diagnostic')
    require(build.get('runner_sha256_before') == build.get('runner_sha256_after') == FINAL_BUILD_RUNNER_SHA256,
            'final-build runner before/after identity mismatch')
    require(build.get('target_absent') is True and execution.get('target_absent_after') is True,
            'final build or diagnostic did not establish target cleanup')
    require(build.get('actual_env_overrides') == {
        'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
        'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
        'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always', 'PROPTEST_RNG_SEED': SEED,
    } and build.get('rust_min_stack') is None, 'final build env/default stack mismatch')
    require(build.get('candidate_byte_identity') is True
            and build.get('final_lint_strict_byte_identity_passed') is True
            and build.get('focused_passed') is True, 'final build did not pass pinned candidate checks')
    cli = build['candidate_cli']
    cli_path = Path(cli['path'])
    require(cli['bytes'] == CLI_BYTES and cli['sha256'] == CLI_SHA256 and cli['mode'] == '0o555',
            'final-build candidate CLI identity fields mismatch')
    require(sha_file(cli_path) == CLI_SHA256 and cli_path.stat().st_size == CLI_BYTES
            and stat.S_IMODE(cli_path.stat().st_mode) == CLI_MODE, 'final-build CLI live identity mismatch')
    require(len(build.get('commands', [])) == 5, 'final-build receipt must bind all five commands')
    for row in build['commands']:
        require(row['exit_code'] == 0 and row['guard_stop'] is None, 'final-build command did not complete cleanly')
        require(row['peak_target_bytes'] <= 1024**3 and row['free_bytes_after'] >= 5 * 1024**3,
                f'final-build resource guard exceeded for command {row["index"]}')
        for stream in row['streams'].values():
            verify_stream(stream)
    return build, pins


def verify_diagnostic(execution_path: Path, runner_path: Path, expected_sha: str, expected_schema: str) -> tuple[dict, list[Path]]:
    require(sha_file(execution_path) == expected_sha, 'execution JSON SHA differs from explicit input pin')
    execution = read_json(execution_path)
    require(execution.get('schema') == expected_schema
            and execution.get('status') == 'completed_same_ir_diagnostic_only',
            'unexpected diagnostic execution schema/status')
    require(Path(execution['runner_path']).resolve() == runner_path.resolve(), 'runner path differs from receipt')
    runner_sha = sha_file(runner_path)
    require(execution.get('runner_sha256_before') == execution.get('runner_sha256_after') == runner_sha,
            'diagnostic runner before/after SHA mismatch')
    require(execution.get('test_path') == EXPECTED_TEST, 'unexpected ignored diagnostic test')
    require(execution.get('normal_closure_scanner_not_run_with_external_include') is True,
            'external include closure handling was not recorded')
    require(execution.get('source_closure_count') == SOURCE_COUNT, 'source closure count mismatch')
    require(execution.get('fresh_target_preflight', {}).get('target_absent') is True,
            'diagnostic did not start with a fresh target')
    guard = execution.get('guard_limits', {})
    require(guard == {'minimum_free_bytes': 5 * 1024**3, 'maximum_target_bytes': 1024**3,
                      'poll_interval_seconds': 1, 'process_group_termination': True},
            'diagnostic guard limits differ from requested values')
    require(execution.get('fresh_target_preflight', {}).get('free_bytes', 0) >= 5 * 1024**3,
            'diagnostic free-space preflight was below 5 GiB')
    require(execution.get('rust_min_stack') is None and execution.get('stripped_environment_keys') == [],
            'diagnostic did not use default Rust stack / clean environment record')
    expected_env = {
        'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
        'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
        'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always', 'PROPTEST_RNG_SEED': SEED,
        'JARDE_FOREACH_CANDIDATE_DIAGNOSTIC': '1',
    }
    require(execution.get('environment_overrides') == expected_env, 'diagnostic environment overrides mismatch')
    commands = execution.get('commands', [])
    require(len(commands) == 2, 'expected exactly test and clean commands')
    test, clean = commands
    expected_test_argv = ['cargo', 'test', '-p', 'jarde', '--lib', '--all-features', '--locked',
                          EXPECTED_TEST, '--', '--ignored', '--exact', '--nocapture']
    require(test['index'] == 0 and test['argv'] == expected_test_argv and test['exit_code'] == 0
            and test['guard_stop'] is None and test['cwd'] == str(ROOT_DEFAULT),
            'exact ignored test invocation did not pass as recorded')
    require(test['env_overrides'] == expected_env, 'test command env differs from receipt')
    require(execution.get('clean_command') == clean and clean['index'] == 1
            and clean['argv'] == ['cargo', 'clean'] and clean['exit_code'] == 0
            and clean['guard_stop'] is None and clean['cwd'] == str(ROOT_DEFAULT),
            'guarded clean command was not the second successful command')
    require(clean['env_overrides'] == expected_env, 'clean command env differs from receipt')
    require(execution.get('target_absent_after') is True, 'target is not recorded absent after clean')
    out = execution_path.parent
    raw_paths = []
    for row in commands:
        require(row['peak_target_bytes'] <= 1024**3 and row['free_bytes_after'] >= 5 * 1024**3,
                f'guard resource limit exceeded for command {row["index"]}')
        for name, ref in row['streams'].items():
            raw = out / f'{row["index"]}.{name}.raw'
            verify_stream(ref, raw)
            raw_paths.append(raw)
    # Tool, candidate and final-build pins must be unchanged across the diagnostic.
    candidate_before = execution.get('candidate_tool_pins_before', {})
    require(execution.get('candidate_tool_pins_match_before') is True
            and candidate_before == execution.get('candidate_tool_pins_after'),
            'candidate/source/tool pins changed during diagnostic')
    for path_text, digest in candidate_before.items():
        require(sha_file(Path(path_text)) == digest, f'live candidate tool pin mismatch: {path_text}')
    verify_build_receipt(execution, ROOT_DEFAULT)
    guard_before = execution['guard_tool_pins_before']
    for path_text, digest in guard_before.items():
        require(sha_file(Path(path_text)) == digest, f'live guard tool pin mismatch: {path_text}')
    require(execution.get('candidate_tool_pins_before', {}).get(str(runner_path)) is None,
            'diagnostic runner unexpectedly appears in candidate tool pins')

    raw_path = out / '0.stderr.raw'
    raw = raw_path.read_text(encoding='utf-8', errors='replace')
    lines = [line for line in raw.splitlines() if 'JARDE_FOREACH_CANDIDATE' in line]
    require(len(lines) >= 4, 'candidate diagnostic event set is incomplete')
    attempt_inner = [x for x in lines if 'event=attempt test_bci=42 header_bci=35 path=[1, 1]' in x]
    attempt_outer = [x for x in lines if 'event=attempt test_bci=13 header_bci=7 path=[1]' in x]
    owner = [x for x in lines if 'event=owner_facts test_bci=42 path=[1, 1]' in x]
    refuse_inner = [x for x in lines if 'event=refused test_bci=42' in x]
    refuse_outer = [x for x in lines if 'event=refused test_bci=13' in x]
    require(len(attempt_inner) == len(attempt_outer) == len(owner) == len(refuse_inner) == len(refuse_outer) == 1,
            'inner/outer owner/refusal diagnostic cardinality mismatch')
    require('body_first=Declare' in attempt_inner[0] and 'form=While' in attempt_inner[0]
            and 'tests=1' in attempt_inner[0], 'inner loop candidate facts mismatch')
    require('init_index=1 init_bci=Some(28) init_store_bci=33 init_slot=Some(5)' in owner[0]
            and 'init_local=Some(LocalVariable { slot: 5, index: 0 })' in owner[0]
            and 'test_local=Some(LocalVariable { slot: 5, index: 0 })' in owner[0]
            and 'next_local=Some(LocalVariable { slot: 5, index: 0 })' in owner[0],
            'init/test/next do not share the recorded slot-5 index-0 LocalVariable')
    require('placement=Some(Elevated { owner: [1] })' in owner[0]
            and 'owner=Some([1])' in owner[0] and 'owner_visible=true' in owner[0]
            and 'emitted=true' in owner[0] and 'owner_at_region=true' in owner[0]
            and 'planned_at_regions=[([1], 33, Reference("java.util.Iterator"))]' in owner[0],
            'ancestor declaration placement/emission proof is incomplete')
    require('gate=post_entry_candidate_proof_incomplete stage=iterator_value_and_phi_use_ownership'
            in refuse_inner[0], 'inner candidate did not record the expected ownership first refusal')
    require('body_first=Assign' in attempt_outer[0]
            and 'gate=body_first_not_initialized_declaration statement=Assign' in refuse_outer[0],
            'outer loop no longer records its original first-Assign refusal')

    snapshot = [x for x in lines if 'event=same_ir_snapshot test_bci=42 path=[1, 1]' in x]
    require(len(snapshot) == 1, 'same-IR snapshot event missing or duplicated')
    snap = snapshot[0]
    require('phis=[(ValueId(12), [Value(ValueId(37)), Itself]' in snap
            and 'replaced_by: Some(ValueId(37))' in snap,
            'header Phi V12 inputs/replacement are not the expected V37 facts')
    tail = snap.split('values_with_uses=', 1)[-1]
    value37 = re.search(r'\(ValueId\(37\), SsaValue \{[^\n]*?uses: \[(.*?)\], replaced_by: None\)', tail)
    require(value37 is not None, 'ValueId(37) use row is absent from same-IR snapshot')
    uses = value37.group(1)
    require(uses.count('CanonicalBlockId { bci: 97, path: [] }, bci: None') == 3,
            'V37 does not have exactly three None-BCI uses at block 97')
    require('CanonicalBlockId { bci: 35, path: [] }, bci: None' in uses
            and 'CanonicalBlockId { bci: 35, path: [] }, bci: Some(35)' in uses
            and 'CanonicalBlockId { bci: 45, path: [] }, bci: Some(45)' in uses,
            'V37 header/body physical and synthetic use facts differ')
    require(execution.get('diagnostic_test_summary') == {'passed': 1, 'failed': 0, 'ignored': 0},
            'one-test summary is not exactly 1/0/0')
    return execution, raw_paths


def archive(execution_path: Path, runner_path: Path, out: Path, execution: dict,
            raw_paths: list[Path], runner_sha: str) -> dict:
    require(not out.exists(), f'refusing to overwrite archive output: {out}')
    require(not out.resolve().is_relative_to(ROOT_DEFAULT), 'archive output must stay outside the shared workspace')
    stage = out.with_name(out.name + '.staging')
    require(not stage.exists(), f'refusing to overwrite staging output: {stage}')
    stage.mkdir(parents=True)
    try:
        shutil.copyfile(execution_path, stage / 'execution.json')
        shutil.copyfile(runner_path, stage / runner_path.name)
        final_path = Path(execution['final_build_receipt']['path'])
        shutil.copyfile(final_path, stage / 'final-build-execution.json')
        final_build = read_json(final_path)
        final_raw = stage / 'final-build-raw'
        final_raw.mkdir()
        for row in final_build['commands']:
            for name, ref in row['streams'].items():
                source = Path(ref['path'])
                shutil.copyfile(source, final_raw / f"{row['index']}.{name}.raw")
        shutil.copyfile(Path(execution['candidate_pins']['path']), stage / 'candidate-pins.json')
        shutil.copyfile(Path(execution['candidate_patch']['path']), stage / 'candidate.patch')
        prep = Path(execution['candidate_prep'])
        candidates = stage / 'candidate-files'
        candidates.mkdir()
        for rel in sorted(CANDIDATE_FILES):
            source = prep / ('build.rs.candidate' if rel.endswith('build.rs') else 'facade.rs.candidate')
            shutil.copyfile(source, candidates / source.name)
        for path_text in execution['guard_tool_pins_before']:
            if path_text.endswith('jarde-constructor-conditional-final-lint-build-root-v1.py'):
                shutil.copyfile(Path(path_text), stage / 'final-build-runner.py')
        raw_dir = stage / 'raw'
        raw_dir.mkdir()
        for raw in raw_paths:
            shutil.copyfile(raw, raw_dir / raw.name)
        result = {
            'status': 'accepted_same_ir_diagnostic_observation_only',
            'execution_sha256': sha_file(execution_path),
            'runner_sha256': runner_sha,
            'final_build_receipt_sha256': sha_file(final_path),
            'raw_streams': [{'name': p.name, 'bytes': p.stat().st_size, 'sha256': sha_file(p)}
                            for p in sorted(raw_paths)],
            'source_closure_count': SOURCE_COUNT,
            'scope': 'One exact ignored diagnostic plus clean; this does not establish a successful foreach candidate or later ownership gate.',
            'first_refusals': {
                'inner_test_bci_42': 'iterator_value_and_phi_use_ownership',
                'outer_test_bci_13': 'body_first_not_initialized_declaration',
            },
        }
        (stage / 'root-acceptance.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n',
                                                       encoding='utf-8')
        entries = []
        for path in sorted(p for p in stage.rglob('*') if p.is_file()):
            entries.append({'path': str(path.relative_to(stage)), 'bytes': path.stat().st_size,
                            'sha256': sha_file(path)})
        (stage / 'archive-inventory.json').write_text(
            json.dumps({'files': entries, 'all_raw_sha_verified_before_copy': True}, indent=2) + '\n',
            encoding='utf-8')
        out.parent.mkdir(parents=True, exist_ok=True)
        stage.rename(out)
        return result
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def main() -> int:
    global ROOT_DEFAULT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execution', type=Path, required=True)
    parser.add_argument('--runner', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=ROOT_DEFAULT)
    parser.add_argument('--expected-execution-sha256', default=EXECUTION_SHA256)
    parser.add_argument('--expected-schema', default='hoisted-iterator-same-ir-diagnostic-root-v1')
    args = parser.parse_args()
    ROOT_DEFAULT = args.root.resolve()
    require(args.execution.is_absolute() and args.runner.is_absolute() and args.out.is_absolute(),
            '--execution, --runner, and --out must be absolute paths')
    execution, raw_paths = verify_diagnostic(args.execution, args.runner,
                                              args.expected_execution_sha256, args.expected_schema)
    result = archive(args.execution, args.runner, args.out, execution, raw_paths,
                     execution['runner_sha256_before'])
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except BaseException as error:
        print(f'acceptance failed: {error}', file=sys.stderr)
        raise SystemExit(1)
