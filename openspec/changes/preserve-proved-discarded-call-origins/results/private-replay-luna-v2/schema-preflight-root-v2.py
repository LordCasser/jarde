from __future__ import annotations

import ast
import hashlib
import json
import re
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = ROOT / 'openspec/changes/preserve-proved-discarded-call-origins/results'
BASE = RESULTS / 'baseline-root-v1'
OUT = Path('/private/tmp/jarde-discarded-call-replay-luna-v2')
EXPECTED_BASE_EXEC = '48a675bf6bf2ffc2c58b8a8ac0e90cad69e5077730a2470ed23f4b52639dbd1e'
EXPECTED_BASE_ACCEPT = 'b2819eb2632925696a117528634468e73f0834ecef28e0d34c248bc82723e717'
BASE_DOC_SHA256 = {
    'javac8/default/class.json': '3b0f64210c612ac1608d33347b4e27489f3025edfeaa504ae42f8fea74ae53f0',
    'javac8/all/class.json': '48f005adce08cd873fa9a23054a4f26c8a4ae8d21a6c66c4e43aaa9c7ae3791a',
    'javac23/default/class.json': '0965dfc297f68098d82b97644f4dcb05dcdca0b89e9d2d8066ecf3ffbc48d139',
    'javac23/all/class.json': 'ce8e474b724b9e24c80048b6ba297612ddaf564a98e4164c35c11bb0d7c3a356',
}
TARGET_GAPS = {
    ('discardStatic', '(Z)V'): 4,
    ('discardAppend', '(Ljava/lang/String;)Ljava/lang/String;'): 13,
    ('discardListAdd', '(Ljava/lang/String;)Ljava/lang/String;'): 15,
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def raw_row(row: dict, stream: str = 'stdout') -> bytes:
    record = row['streams'][stream]
    path = Path(record['path'])
    path = path if path.is_absolute() else ROOT / path
    data = path.read_bytes()
    assert len(data) == record['bytes'] and sha(data) == record['sha256']
    return data


def bcis(raw: bytes) -> dict[tuple[str, str], set[int]]:
    result = {}
    name = None
    current = None
    for line in raw.decode('utf-8').splitlines():
        if re.match(r'^  (public|private|protected) .*(?:\);|\);)$', line):
            header = line.strip().split('(')[0].split()[-1]
            name = '<init>' if '.' in header else header
            current = None
        elif name and (match := re.match(r'^    descriptor: (.*)$', line)):
            current = result.setdefault((name, match[1]), set())
        elif current is not None and (match := re.match(r'^\s+(\d+): [a-z][a-z0-9_]*(?:\s|$)', line)):
            current.add(int(match[1]))
    return result


def main() -> None:
    collector = OUT / 'collect-candidate-root-v2.py'
    verifier = OUT / 'verify-candidate-root-v2.py'
    ast.parse(collector.read_text(encoding='utf-8'))
    ast.parse(verifier.read_text(encoding='utf-8'))
    exec_path = BASE / 'execution.json'
    acceptance_path = BASE / 'acceptance-root-v1.json'
    exec_raw = exec_path.read_bytes()
    accept_raw = acceptance_path.read_bytes()
    assert sha(exec_raw) == EXPECTED_BASE_EXEC
    assert sha(accept_raw) == EXPECTED_BASE_ACCEPT
    execution = json.loads(exec_raw)
    acceptance = json.loads(accept_raw)
    assert execution['schema'] == 'discarded-call-source-baseline-root-v1'
    assert execution['status'] == 'observed-baseline-not-accepted'
    assert acceptance['schema'] == 'discarded-call-source-baseline-root-acceptance-v1'
    assert acceptance['status'] == 'accepted-baseline-observations-only'
    assert acceptance['manifest_sha256'] == EXPECTED_BASE_EXEC
    assert acceptance['expected_missing_pop_bcis'] == {
        'discardStatic': [4], 'discardAppend': [13], 'discardListAdd': [15],
    }
    assert len(execution['commands']) == 26
    by_label = {row['label']: row for row in execution['commands']}
    assert len(by_label) == 26
    for row in execution['commands']:
        assert row['exit_code'] == 0 and row['guard_stop'] is None
        raw_row(row, 'stdout')
        raw_row(row, 'stderr')
    assert execution['legs']['javac23'].keys() == {'tools', 'profiles', 'original_classes', 'original_runtime_index'}
    assert execution['legs']['javac23']['profiles']['default'].keys() == {'render_index', 'compile_index', 'runtime_index'}
    assert set(execution['legs']['javac23']['original_classes']) == {
        'discardprobe/DiscardedCallSourceProbe.class', 'discardprobe/DiscardedCallSourceProbeRunner.class', 'discardprobe/ProbePop2.class',
    }
    all_profiles = {}
    report_keys = set()
    method_entry_keys = set()
    origin_keys = set()
    report_method_values = {}
    derived_example = None
    missing_by_profile = {}
    for leg in ('javac8', 'javac23'):
        row = by_label[leg + '-DiscardedCallSourceProbe-javap']
        physical = bcis(raw_row(row))
        assert len(physical) == 7
        assert set(physical) == {
            ('<init>', '()V'), ('give', '(Z)Ljava/lang/String;'), ('discardStatic', '(Z)V'),
            ('discardAppend', '(Ljava/lang/String;)Ljava/lang/String;'),
            ('discardListAdd', '(Ljava/lang/String;)Ljava/lang/String;'),
            ('consumeReturn', '()Ljava/lang/String;'), ('deferToLocal', '()Ljava/lang/String;'),
        }
        for profile in ('default', 'all'):
            rel_doc = f'{leg}/{profile}/class.json'
            doc_raw = (BASE / rel_doc).read_bytes()
            assert sha(doc_raw) == BASE_DOC_SHA256[rel_doc]
            doc = json.loads(doc_raw)
            class_bytes = (BASE / leg / 'original/discardprobe/DiscardedCallSourceProbe.class').read_bytes()
            assert doc['class']['class_bytes']['length'] == len(class_bytes)
            assert sha(class_bytes) == execution['legs'][leg]['original_classes']['discardprobe/DiscardedCallSourceProbe.class']
            all_profiles[leg + '/' + profile] = doc
            assert 'methods' in doc and len(doc['methods']) == 7
            doc_method_ids = set()
            missing = {}
            for entry in doc['methods']:
                method_entry_keys |= set(entry)
                item = entry['item']
                assert 'identity' not in entry and 'identity' in item
                identity = item['identity']
                assert identity['owner']['class_bytes'] == doc['class']['class_bytes']
                key = (item['name']['escaped'], item['descriptor']['escaped'])
                doc_method_ids.add(key)
                outcome = entry['outcome']
                assert outcome['kind'] == 'recovered'
                report = outcome['report']
                report_keys |= set(report)
                assert report['method'] == key[0] + key[1] and isinstance(report['method'], str)
                assert isinstance(report['fallbacks'], list)
                assert isinstance(entry['text'], str)
                assert report['artifact']['binding']['method'] == identity
                assert report['source_map'].keys() == {'segments'}
                covered = set()
                text_bytes = report['text'].encode('utf-8')
                for segment in report['source_map']['segments']:
                    assert segment.keys() == {'start', 'end', 'origin'}
                    assert 0 <= segment['start'] <= segment['end'] <= len(text_bytes)
                    for origin in [segment['origin']['primary'], *segment['origin']['derived']]:
                        origin_keys |= set(origin)
                        assert origin['method'] == identity
                        covered.add(origin['bci'])
                        if origin['provenance'] == 'Derived':
                            derived_example = origin
                gap = physical[key] - covered
                if gap:
                    missing[key] = gap
                assert not covered - physical[key]
            assert doc_method_ids == set(physical)
            for target_key, pop_bci in TARGET_GAPS.items():
                fixed = {
                    ('discardStatic', '(Z)V'): ((194, 210), 1),
                    ('discardAppend', '(Ljava/lang/String;)Ljava/lang/String;'): ((296, 321), 10),
                    ('discardListAdd', '(Ljava/lang/String;)Ljava/lang/String;'): ((289, 330), 10),
                }[target_key]
                method = next(row for row in doc['methods']
                              if (row['item']['name']['escaped'], row['item']['descriptor']['escaped']) == target_key)
                spans = [(segment['start'], segment['end'], segment['origin']['primary']['bci'])
                         for segment in method['outcome']['report']['source_map']['segments']]
                assert spans.count((*fixed[0], fixed[1])) == 1
            expected = {(name, desc): {bci} for (name, desc), bci in TARGET_GAPS.items()}
            assert missing == expected, (leg, profile, missing)
            missing_by_profile[leg + '/' + profile] = {k[0]: sorted(v) for k, v in missing.items()}
        assert all_profiles[leg + '/default']['text'] == all_profiles[leg + '/all']['text']
        assert {m['item']['name']['escaped']: m['outcome']['report']['source_map'] for m in all_profiles[leg + '/default']['methods']} == {
            m['item']['name']['escaped']: m['outcome']['report']['source_map'] for m in all_profiles[leg + '/all']['methods']
        }
    assert {'bci', 'cp', 'method', 'provenance'} <= origin_keys
    assert derived_example is not None and derived_example['cp'] is None and derived_example['provenance'] == 'Derived'
    assert method_entry_keys == {'annotations', 'declaration', 'item', 'markers', 'no_body_kind', 'outcome', 'parameter_annotations', 'text', 'type_annotations'}
    assert 'method' in report_keys and 'fallbacks' in report_keys and 'source_map' in report_keys
    runner_path = Path('/private/tmp/jarde-discarded-call-validation-luna-v1/run-validation-build-root-v1.py')
    runner = runner_path.read_text(encoding='utf-8')
    for contract in (
        'preserve-proved-discarded-call-origins-validation-build-root-v1',
        'validation_runner', 'guarded_runner_template', 'source_commit_base_expected',
        'uncommitted_discarded_call_product', 'expected_origin_test_count',
        'expected_library_test_count', 'product_path_sets',
        'preserve-proved-discarded-call-origins-candidate-cli-v1',
        '/private/tmp/jarde-proved-discarded-call-cli-v1',
    ):
        assert contract in runner, contract
    runner_ast = ast.parse(runner)
    write_exec = next(node for node in runner_ast.body if isinstance(node, ast.FunctionDef) and node.name == 'write_execution')
    write_dict = next(node.args[1] for node in ast.walk(write_exec)
                      if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
                      and node.func.attr == 'write_json' and len(node.args) >= 2
                      and isinstance(node.args[1], ast.Dict))
    execution_fields = {key.value for key in write_dict.keys if isinstance(key, ast.Constant)}
    assert execution_fields == {
        'schema', 'validation_runner', 'guarded_runner_template', 'status', 'source_commit_base_expected',
        'uncommitted_discarded_call_product', 'environment_overrides', 'required_origin_tests',
        'expected_origin_test_count', 'expected_library_test_count', 'guards', 'preflight', 'commands', 'freeze',
    }
    main_node = next(node for node in runner_ast.body if isinstance(node, ast.FunctionDef) and node.name == 'main')
    dicts = [node.value for node in ast.walk(main_node) if isinstance(node, ast.Assign)
             and any(isinstance(target, ast.Name) and target.id in ('freeze', 'metadata') for target in node.targets)
             and isinstance(node.value, ast.Dict)]
    named_dict_fields = {}
    for node in ast.walk(main_node):
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id in ('freeze', 'metadata') for target in node.targets) and isinstance(node.value, ast.Dict):
            keyset = {key.value for key in node.value.keys if isinstance(key, ast.Constant)}
            for target in node.targets:
                if isinstance(target, ast.Name):
                    named_dict_fields[target.id] = keyset
    assert named_dict_fields['freeze'] == {
        'cli_path', 'cli_sha256', 'cli_mode', 'metadata_path', 'source_commit_base',
        'uncommitted_discarded_call_product', 'validation_runner', 'guarded_runner_template',
        'product_path_sets', 'required_origin_tests', 'expected_origin_test_count', 'expected_library_test_count',
    }
    assert named_dict_fields['metadata'] == {
        'schema', 'candidate_sources', 'test_sources', 'canonical_files', 'build_result_sha256',
    }
    output = {
        'schema': 'discarded-call-replay-json-schema-preflight-root-v2',
        'status': 'pure-json-schema-assertions-passed',
        'baseline_execution_sha256': sha(exec_raw),
        'baseline_acceptance_sha256': sha(accept_raw),
        'baseline_class_json_sha256': BASE_DOC_SHA256,
        'baseline_commands': len(execution['commands']),
        'baseline_methods_per_profile': 7,
        'baseline_missing_by_profile': missing_by_profile,
        'actual_method_entry_identity_path': 'methods[].item.identity',
        'actual_report_method_type': 'string name+descriptor',
        'actual_report_has_fallbacks': True,
        'actual_entry_text_type': 'string',
        'actual_origin_fields': sorted(origin_keys),
        'derived_origin_sample': derived_example,
        'validation_runner_path': str(runner_path),
        'validation_cli_path': '/private/tmp/jarde-proved-discarded-call-cli-v1',
        'collector_ast': 'passed', 'verifier_ast': 'passed',
        'toolchain_executed': False,
    }
    (OUT / 'schema-preflight-root-v2.json').write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(output, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
