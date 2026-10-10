from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import stat
import sys

from blake3 import blake3

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE = ROOT / 'openspec/changes/preserve-proved-discarded-call-origins'
RESULTS = CHANGE / 'results'
BASE = RESULTS / 'baseline-root-v1'
INPUT = RESULTS / 'original-inputs-root-v1'
OUT = RESULTS / 'candidate-replay-root-v2'
BASE_EXEC = BASE / 'execution.json'
BASE_ACCEPT = BASE / 'acceptance-root-v1.json'
BASE_MANIFEST_SHA256 = '48a675bf6bf2ffc2c58b8a8ac0e90cad69e5077730a2470ed23f4b52639dbd1e'
BASE_ACCEPT_SHA256 = 'b2819eb2632925696a117528634468e73f0834ecef28e0d34c248bc82723e717'
BASE_INPUTS = {
    'DiscardedCallSourceProbe.java': '2f067f8110639ab68467a397c21824eabb8bc434cd1376c8a57369d2ffbdf1ea',
    'DiscardedCallSourceProbeRunner.java': '180f520e94f7250fbbf0a96699376e5878cdc8418e78fa4a90cf738aee5155b5',
    'ProbePop2.java': '914f126077bf34d9551ce9afd9b4f66eb0033f059a9d158da0b3092dcf68d630',
}
BASE_DOC_SHA256 = {
    'javac8/default/class.json': '3b0f64210c612ac1608d33347b4e27489f3025edfeaa504ae42f8fea74ae53f0',
    'javac8/all/class.json': '48f005adce08cd873fa9a23054a4f26c8a4ae8d21a6c66c4e43aaa9c7ae3791a',
    'javac23/default/class.json': '0965dfc297f68098d82b97644f4dcb05dcdca0b89e9d2d8066ecf3ffbc48d139',
    'javac23/all/class.json': 'ce8e474b724b9e24c80048b6ba297612ddaf564a98e4164c35c11bb0d7c3a356',
}
REQUIRED_ORIGIN_TESTS = {
    'frozen_probe_has_exact_identity',
    'proved_static_virtual_and_interface_call_pops_map_to_complete_statements',
    'consumed_and_local_deferred_calls_keep_their_existing_body_and_map',
    'exact_pop_charge_and_cancel_publish_no_partial_statement',
    'wide_pop2_is_not_attached_to_a_call_statement',
}
JDKS = {
    'javac8': {
        'home': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
        'tools': {
            'java': '3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5',
            'javac': '598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd',
            'javap': 'fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1',
        },
    },
    'javac23': {
        'home': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
        'tools': {
            'java': 'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1',
            'javac': 'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8',
            'javap': 'f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e',
        },
    },
}
JADX = Path('/opt/homebrew/bin/jadx')
JADX_SHA256 = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
EXPECTED_STDOUT = (
    b'static-normal=completed\nstatic-throw=give-failed\nappend=appended\n'
    b'list-add=listed\nreturn-consumed=given\nlocal-deferred=given\n'
)
METHODS = {
    ('discardStatic', '(Z)V'): {'pop': 4, 'span': (194, 210), 'primary': 1},
    ('discardAppend', '(Ljava/lang/String;)Ljava/lang/String;'): {'pop': 13, 'span': (296, 321), 'primary': 10},
    ('discardListAdd', '(Ljava/lang/String;)Ljava/lang/String;'): {'pop': 15, 'span': (289, 330), 'primary': 10},
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_raw(row: dict, key: str = 'stdout') -> bytes:
    rec = row['streams'][key]
    path = Path(rec['path'])
    path = path if path.is_absolute() else ROOT / path
    data = path.read_bytes()
    assert len(data) == rec['bytes'] and sha(data) == rec['sha256'], (row['label'], key)
    return data


def method_bcis(data: bytes) -> dict[tuple[str, str], set[int]]:
    result: dict[tuple[str, str], set[int]] = {}
    name = None
    current = None
    for line in data.decode('utf-8', errors='strict').splitlines():
        if re.match(r'^  (public|private|protected) .*(?:\);|\);)$', line):
            header = line.strip().split('(')[0].split()[-1]
            name = '<init>' if '.' in header else header
            current = None
        elif name and (match := re.match(r'^    descriptor: (.*)$', line)):
            current = result.setdefault((name, match[1]), set())
        elif current is not None and (match := re.match(r'^\s+(\d+): [a-z][a-z0-9_]*(?:\s|$)', line)):
            current.add(int(match[1]))
    return result


def inventory(directory: Path) -> dict[str, dict[str, object]]:
    result = {}
    for path in sorted(directory.rglob('*.class')):
        data = path.read_bytes()
        result[path.relative_to(directory).as_posix()] = {
            'length': len(data), 'sha256': sha(data), 'blake3': blake3(data).hexdigest(),
        }
    return result


def map_sources(doc: dict, physical: dict[tuple[str, str], set[int]], class_identity: dict,
                allowed_missing: dict[tuple[str, str], set[int]]) -> dict:
    result = {}
    methods = doc['methods']
    for entry in methods:
        item = entry['item']
        key = (item['name']['escaped'], item['descriptor']['escaped'])
        outcome = entry['outcome']
        assert outcome['kind'] == 'recovered', key
        report = outcome['report']
        pid = item['identity']
        assert report['artifact']['binding']['method'] == pid
        assert report['method'] == key[0] + key[1]
        assert pid['owner']['class_bytes'] == class_identity
        assert bytes(pid['name']).decode() == key[0] and bytes(pid['descriptor']).decode() == key[1]
        assert report['fallbacks'] == []
        text = report['text'].encode('utf-8')
        covered = set()
        for segment in report['source_map']['segments']:
            assert 0 <= segment['start'] <= segment['end'] <= len(text)
            for origin in [segment['origin']['primary'], *segment['origin']['derived']]:
                assert origin['method'] == pid
                covered.add(origin['bci'])
        missing = physical[key] - covered
        assert missing == allowed_missing.get(key, set()), (key, sorted(missing), sorted(covered - physical[key]))
        assert not (covered - physical[key]), (key, sorted(covered - physical[key]))
        result[key] = {'pid': pid, 'report': report, 'entry': entry}
    assert set(result) == set(physical) and len(result) == 7
    return result


def expected_command_rows(cli: Path, legs: dict) -> list[tuple[str, list[str]]]:
    expected = []
    for leg in ('javac8', 'javac23'):
        home = JDKS[leg]['home']
        d = OUT / leg
        original = d / 'original'
        t = {name: home / 'bin' / name for name in ('java', 'javac', 'javap')}
        expected.append((leg + '-original-compile', [str(t['javac']), '-encoding', 'UTF-8', '-g:none', '-source', '8', '-target', '8', '-proc:none', '-implicit:none', '-d', str(original), str(INPUT / 'DiscardedCallSourceProbe.java'), str(INPUT / 'DiscardedCallSourceProbeRunner.java'), str(INPUT / 'ProbePop2.java')]))
        expected.append((leg + '-DiscardedCallSourceProbe-javap', [str(t['javap']), '-p', '-c', '-v', '-classpath', str(original), 'discardprobe.DiscardedCallSourceProbe']))
        expected.append((leg + '-ProbePop2-javap', [str(t['javap']), '-p', '-c', '-v', '-classpath', str(original), 'discardprobe.ProbePop2']))
        expected.append((leg + '-original-runtime', [str(t['java']), '-Xverify:all', '-cp', str(original), 'discardprobe.DiscardedCallSourceProbeRunner']))
        for profile in ('default', 'all', 'jadx'):
            out = d / profile
            src = out / 'sources'
            compiled = out / 'classes'
            source = src / 'DiscardedCallSourceProbe.java'
            runner = src / 'DiscardedCallSourceProbeRunner.java'
            if profile == 'jadx':
                render = [str(JADX), '--no-res', '--config', 'none', '--threads-count', '1', '-d', str(out / 'decompile'), str(original / 'discardprobe/DiscardedCallSourceProbe.class')]
                label = leg + '-jadx-render'
            else:
                render = [str(cli), 'class-source', '--input', str(original / 'discardprobe/DiscardedCallSourceProbe.class'), '--class', 'discardprobe.DiscardedCallSourceProbe', '--policy', 'single-class', '--release', '8', '--format', 'json']
                if profile == 'all':
                    render += ['--evidence', 'all']
                label = leg + '-' + profile + '-render'
            expected.append((label, render))
            expected.append((leg + '-' + profile + '-compile', [str(t['javac']), '-encoding', 'UTF-8', '-g:none', '-source', '8', '-target', '8', '-proc:none', '-implicit:none', '-classpath', str(compiled), '-d', str(compiled), str(source), str(runner)]))
            expected.append((leg + '-' + profile + '-runtime', [str(t['java']), '-Xverify:all', '-cp', str(compiled), 'discardprobe.DiscardedCallSourceProbeRunner']))
    return expected


def main() -> int:
    parser = argparse.ArgumentParser(description='Independently verify the discarded-call complete-class candidate replay.')
    parser.add_argument('--cli', required=True)
    parser.add_argument('--metadata', required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--source-base', required=True)
    args = parser.parse_args()
    cli = Path(args.cli).resolve(strict=True)
    metadata_path = Path(args.metadata).resolve(strict=True)
    build_path = Path(args.build).resolve(strict=True)
    assert cli == Path('/private/tmp/jarde-proved-discarded-call-cli-v1')
    assert metadata_path == (RESULTS / 'candidate-cli-v1.json').resolve()
    assert build_path == (RESULTS / 'validation-build-root-v1/execution.json').resolve()
    execution_path = OUT / 'execution.json'
    candidate = json.loads(execution_path.read_bytes())
    assert candidate['schema'] == 'discarded-call-candidate-replay-root-v2'
    assert candidate['status'] == 'candidate-replay-observed'
    assert sha(Path(candidate['collector_path']).read_bytes()) == candidate['collector_sha256']
    assert candidate['candidate_cli'] == {'path': str(cli), 'sha256': sha(cli.read_bytes()), 'mode': '0555'}
    assert stat.S_IMODE(cli.stat().st_mode) == 0o555
    assert candidate['metadata_path'] == str(metadata_path) and candidate['metadata_sha256'] == sha(metadata_path.read_bytes())
    assert candidate['build_result_path'] == str(build_path) and candidate['build_result_sha256'] == sha(build_path.read_bytes())
    assert candidate['source_commit_base'] == args.source_base.lower()
    metadata = json.loads(metadata_path.read_bytes())
    build = json.loads(build_path.read_bytes())
    assert metadata['schema'] == 'preserve-proved-discarded-call-origins-candidate-cli-v1'
    assert metadata['source_commit_base'] == args.source_base.lower()
    assert metadata['cli_path'] == str(cli) and metadata['cli_sha256'] == sha(cli.read_bytes())
    assert metadata['cli_mode'] == '0o555' and stat.S_IMODE(cli.stat().st_mode) == 0o555
    assert metadata['build_result_sha256'] == sha(build_path.read_bytes())
    assert build['schema'] == 'preserve-proved-discarded-call-origins-validation-build-root-v1'
    assert build['status'] == 'validation-passed-cli-frozen'
    assert build['source_commit_base_expected'] == args.source_base.lower()
    assert build['preflight']['git_head']['matches_expected'] is True
    assert build['preflight']['git_head']['value'] == args.source_base.lower()
    assert build['uncommitted_discarded_call_product'] is True
    runner_path = Path(build['validation_runner']['path'])
    assert runner_path == RESULTS / 'run-validation-build-root-v1.py'
    assert sha(runner_path.read_bytes()) == build['validation_runner']['sha256']
    assert build['freeze']['validation_runner'] == build['validation_runner']
    assert build['guarded_runner_template'] == build['freeze']['guarded_runner_template']
    assert build['guarded_runner_template']['path'] == str(ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py')
    assert build['guarded_runner_template']['sha256'] == '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
    guard_path = Path(build['guarded_runner_template']['path'])
    assert sha(guard_path.read_bytes()) == build['guarded_runner_template']['sha256']
    assert build['preflight']['git_head']['matches_expected'] is True
    assert build['preflight']['git_head']['value'] == args.source_base.lower()
    assert build['freeze']['cli_path'] == str(cli) and build['freeze']['cli_sha256'] == sha(cli.read_bytes())
    assert build['freeze']['metadata_path'] == str(metadata_path)
    pins = build['preflight']['source_pins_after']
    assert pins == build['preflight']['source_pins_before']
    for group in ('candidate_sources', 'test_sources', 'canonical_files'):
        assert metadata[group] == pins[group]
        assert build['freeze']['product_path_sets'][group] == sorted(pins[group])
        for relative, expected in pins[group].items():
            path = ROOT / relative
            assert path.is_file() and sha(path.read_bytes()) == expected, relative
    for field in ('required_origin_tests', 'expected_origin_test_count', 'expected_library_test_count'):
        assert metadata[field] == build[field] == build['freeze'][field]
    assert metadata['expected_origin_test_count'] == len(metadata['required_origin_tests']) == 5
    assert set(metadata['required_origin_tests']) == REQUIRED_ORIGIN_TESTS
    assert build['validation_runner']['sha256'] == build['freeze']['validation_runner']['sha256']
    baseline_exec = BASE_EXEC.read_bytes()
    baseline_accept = json.loads(BASE_ACCEPT.read_bytes())
    assert sha(baseline_exec) == BASE_MANIFEST_SHA256 and sha(BASE_ACCEPT.read_bytes()) == BASE_ACCEPT_SHA256
    assert baseline_accept['schema'] == 'discarded-call-source-baseline-root-acceptance-v1'
    assert baseline_accept['status'] == 'accepted-baseline-observations-only'
    assert baseline_accept['manifest_sha256'] == BASE_MANIFEST_SHA256
    old = json.loads(baseline_exec)
    assert old['status'] == 'observed-baseline-not-accepted' and len(old['commands']) == 26
    assert old['cli']['path'] == '/private/tmp/jarde-return-arm-latch-cli-v1'
    assert old['cli']['sha256'] != sha(cli.read_bytes())
    old_by_label = {row['label']: row for row in old['commands']}
    assert len(old_by_label) == 26
    for row in old['commands']:
        assert row['exit_code'] == 0 and row['guard_stop'] is None
        for stream in row['streams'].values():
            raw_path = Path(stream['path'])
            raw_path = raw_path if raw_path.is_absolute() else ROOT / raw_path
            data = raw_path.read_bytes()
            assert len(data) == stream['bytes'] and sha(data) == stream['sha256'], (row['label'], stream)
    for leg, leg_data in old['legs'].items():
        for relative, expected in leg_data['original_classes'].items():
            assert sha((BASE / leg / 'original' / relative).read_bytes()) == expected
    assert candidate['baseline_class_json_sha256'] == BASE_DOC_SHA256
    for relative, expected in BASE_DOC_SHA256.items():
        assert sha((BASE / relative).read_bytes()) == expected
    assert candidate['baseline_execution_sha256'] == BASE_MANIFEST_SHA256
    assert candidate['baseline_acceptance_sha256'] == sha(BASE_ACCEPT.read_bytes())
    for name, expected in BASE_INPUTS.items():
        path = INPUT / name
        assert sha(path.read_bytes()) == expected == old['inputs'][name]
        assert candidate['original_inputs'][name] == expected
    assert sha(JADX.read_bytes()) == JADX_SHA256 and candidate['jadx'] == {'path': str(JADX), 'sha256': JADX_SHA256}
    assert candidate['guard']['template_sha256'] == '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
    assert len(candidate['commands']) == 26
    by_label = {row['label']: row for row in candidate['commands']}
    assert len(by_label) == 26
    expected_rows = expected_command_rows(cli, candidate['legs'])
    assert len(expected_rows) == 26
    for index, ((label, argv), row) in enumerate(zip(expected_rows, candidate['commands'])):
        assert row['index'] == index and row['label'] == label and row['argv'] == argv
        assert row['cwd'] == str(ROOT) and row['exit_code'] == 0 and row['guard_stop'] is None
        for stream in ('stdout', 'stderr'):
            read_raw(row, stream)
    all_classes = 0
    runtime_pairs = []
    for leg, info in JDKS.items():
        leg_data = candidate['legs'][leg]
        expected_tools = {name: {'path': str(info['home'] / 'bin' / name), 'sha256': info['tools'][name]} for name in ('java', 'javac', 'javap')}
        assert leg_data['tools'] == expected_tools
        for name, tool in expected_tools.items():
            assert sha(Path(tool['path']).read_bytes()) == tool['sha256']
        orig_dir = OUT / leg / 'original'
        original_inventory = inventory(orig_dir)
        assert set(original_inventory) == {'discardprobe/DiscardedCallSourceProbe.class', 'discardprobe/DiscardedCallSourceProbeRunner.class', 'discardprobe/ProbePop2.class'}
        assert leg_data['original']['classes'] == original_inventory
        assert original_inventory == {
            name: {'length': (orig_dir / name).stat().st_size, 'sha256': digest, 'blake3': blake3((orig_dir / name).read_bytes()).hexdigest()}
            for name, digest in old['legs'][leg]['original_classes'].items()
        }
        all_classes += len(original_inventory)
        for profile in ('default', 'all', 'jadx'):
            profile_data = leg_data['profiles'][profile]
            classes_dir = OUT / leg / profile / 'classes'
            class_set = inventory(classes_dir)
            assert set(class_set) == {'discardprobe/DiscardedCallSourceProbe.class', 'discardprobe/DiscardedCallSourceProbeRunner.class'}
            assert profile_data['classes'] == class_set
            all_classes += len(class_set)
            source = OUT / leg / profile / 'sources/DiscardedCallSourceProbe.java'
            runner_source = OUT / leg / profile / 'sources/DiscardedCallSourceProbeRunner.java'
            assert sha(source.read_bytes()) == profile_data['generated_source_sha256']
            assert sha(runner_source.read_bytes()) == profile_data['runner_sha256'] == BASE_INPUTS['DiscardedCallSourceProbeRunner.java']
            if profile in ('default', 'all'):
                class_json = OUT / leg / profile / 'class.json'
                assert class_json.read_bytes() == read_raw(by_label[leg + '-' + profile + '-render'])
                doc = json.loads(class_json.read_bytes())
                assert doc['text'].encode() == source.read_bytes()
                assert doc['text'] == json.loads((BASE / leg / profile / 'class.json').read_bytes())['text']
            else:
                rendered_sources = list((OUT / leg / profile / 'decompile').rglob('DiscardedCallSourceProbe.java'))
                assert len(rendered_sources) == 1 and rendered_sources[0].read_bytes() == source.read_bytes()
            oracle = by_label[leg + '-original-runtime']
            replay = by_label[leg + '-' + profile + '-runtime']
            assert read_raw(oracle) == EXPECTED_STDOUT and read_raw(oracle, 'stderr') == b''
            assert read_raw(replay) == EXPECTED_STDOUT and read_raw(replay, 'stderr') == b''
            assert (read_raw(replay), read_raw(replay, 'stderr')) == (read_raw(oracle), read_raw(oracle, 'stderr'))
            runtime_pairs.append((read_raw(replay), read_raw(replay, 'stderr')))
        raw_probe = read_raw(by_label[leg + '-DiscardedCallSourceProbe-javap'])
        physical = method_bcis(raw_probe)
        assert len(physical) == 7
        # Compare every method/descriptor/instruction BCI to the immutable javac baseline.
        old_javap = old_by_label[leg + '-DiscardedCallSourceProbe-javap']
        assert physical == method_bcis(read_raw(old_javap))
        class_file = orig_dir / 'discardprobe/DiscardedCallSourceProbe.class'
        class_bytes = class_file.read_bytes()
        class_identity = {'digest': blake3(class_bytes).hexdigest(), 'length': len(class_bytes)}
        baseline_docs = {}
        candidate_docs = {}
        for profile in ('default', 'all'):
            baseline_doc = json.loads((BASE / leg / profile / 'class.json').read_bytes())
            candidate_doc = json.loads((OUT / leg / profile / 'class.json').read_bytes())
            assert baseline_doc['class']['class_bytes'] == class_identity == candidate_doc['class']['class_bytes']
            baseline_docs[profile] = baseline_doc
            candidate_docs[profile] = candidate_doc
        assert baseline_docs['default']['text'] == baseline_docs['all']['text']
        assert baseline_docs['default']['text'] == candidate_docs['default']['text'] == candidate_docs['all']['text']
        old_gaps = {
            ('discardStatic', '(Z)V'): {4},
            ('discardAppend', '(Ljava/lang/String;)Ljava/lang/String;'): {13},
            ('discardListAdd', '(Ljava/lang/String;)Ljava/lang/String;'): {15},
        }
        assert baseline_accept['expected_missing_pop_bcis'] == {
            'discardStatic': [4], 'discardAppend': [13], 'discardListAdd': [15],
        }
        candidate_gaps = {}
        old_default = map_sources(baseline_docs['default'], physical, class_identity, old_gaps)
        old_all = map_sources(baseline_docs['all'], physical, class_identity, old_gaps)
        new_default = map_sources(candidate_docs['default'], physical, class_identity, candidate_gaps)
        new_all = map_sources(candidate_docs['all'], physical, class_identity, candidate_gaps)
        assert {key: value['report']['source_map'] for key, value in old_default.items()} == {key: value['report']['source_map'] for key, value in old_all.items()}
        assert {key: value['report']['source_map'] for key, value in new_default.items()} == {key: value['report']['source_map'] for key, value in new_all.items()}
        for key in physical:
            before = old_default[key]
            after = new_default[key]
            assert before['pid'] == after['pid'], key
            assert before['entry']['text'] == after['entry']['text']
            assert before['report']['text'] == after['report']['text']
            assert before['report']['fallbacks'] == after['report']['fallbacks'] == []
            old_segments = before['report']['source_map']['segments']
            new_segments = copy.deepcopy(after['report']['source_map']['segments'])
            assert len(old_segments) == len(new_segments)
            target = METHODS.get(key)
            if target is None:
                assert new_segments == old_segments, key
                continue
            span = target['span']
            primary_bci = target['primary']
            matches = [i for i, segment in enumerate(old_segments)
                       if (segment['start'], segment['end'], segment['origin']['primary']['bci']) == (*span, primary_bci)]
            assert len(matches) == 1, (key, matches)
            index = matches[0]
            expected_origin = {
                'bci': target['pop'], 'cp': None, 'method': after['pid'], 'provenance': 'Derived',
            }
            assert expected_origin not in old_segments[index]['origin']['derived']
            added = [origin for origin in new_segments[index]['origin']['derived']
                     if origin not in old_segments[index]['origin']['derived']]
            assert added == [expected_origin], (key, added)
            new_segments[index]['origin']['derived'].remove(expected_origin)
            assert new_segments == old_segments, key
        # Required runtime profiles and the two javac original oracles are byte-identical.
    assert all(pair == runtime_pairs[0] for pair in runtime_pairs)
    assert all_classes == 18
    result = {
        'schema': 'discarded-call-candidate-replay-acceptance-root-v2',
        'status': 'accepted-candidate-replay',
        'candidate_execution_sha256': sha(execution_path.read_bytes()),
        'candidate_cli_sha256': sha(cli.read_bytes()),
        'metadata_sha256': sha(metadata_path.read_bytes()),
        'build_result_sha256': sha(build_path.read_bytes()),
        'baseline_execution_sha256': BASE_MANIFEST_SHA256,
        'commands': 26, 'complete_runtime_legs': 8, 'method_profiles': 28,
        'bound_class_artifacts': all_classes,
        'exact_derived_pop_deltas': {name: [data['pop']] for (name, _), data in METHODS.items()},
        'retained_gaps': [], 'candidate_fix_accepted': True,
    }
    target = OUT / 'acceptance-root-v2.json'
    target.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
