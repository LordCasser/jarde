#!/usr/bin/env python3
"""Independent read-only verifier for recovered 18-case GC09 field inputs."""
import collections
import hashlib
import json
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/candidate/field18-msrv-final/complete-run'
MANIFEST = OUT / 'manifest.json'
PREFLIGHT = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/field18-msrv-final-preflight.json'
HIST = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/candidate/gc09-field-23-v9/adapter-run/run-metadata.json'
CLI_META = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/local-gates/candidate-cli-v9.json'
LINKAGE = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/msrv-output-equivalence/cli9-replay-20261009/field18-recovery-linkage.json'
REPORT = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/field18-msrv-final-verification.json'
EXPECTED_CANDIDATE_SHA = '5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006'
EXPECTED_BASELINE_SHA = '3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70'
EXPECTED_ARCHIVE_SHA = 'fc4220ac5eb6906a786340a6ef5f8dd840ccbc14790df7bcee115f7c9f78fce8'
EXPECTED_LINKAGE_SHA = 'e844416805d608e9e5203824553d56841108bd00e167bb914698349f2a6aef50'


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def main():
    m = json.loads(MANIFEST.read_text())
    pre = json.loads(PREFLIGHT.read_text())
    old = json.loads(HIST.read_text())
    cli = json.loads(CLI_META.read_text())
    linkage = json.loads(LINKAGE.read_text())
    require(pre['recovery_manifest_sha256'] == sha(MANIFEST), 'preflight manifest hash mismatch')
    require(m['candidate_cli_sha256'] == EXPECTED_CANDIDATE_SHA, 'candidate CLI identity mismatch')
    require(m['baseline_cli_sha256'] == EXPECTED_BASELINE_SHA, 'baseline CLI identity mismatch')
    require(cli['sha256'] == EXPECTED_CANDIDATE_SHA, 'candidate metadata CLI SHA mismatch')
    require(cli['source_archive_sha256'] == EXPECTED_ARCHIVE_SHA, 'source archive identity mismatch')
    require(cli['path'] == m['candidate_cli'], 'candidate path mismatch')
    require(sha(m['candidate_cli']) == EXPECTED_CANDIDATE_SHA, 'candidate executable bytes mismatch')
    require(sha(m['baseline_cli']) == EXPECTED_BASELINE_SHA, 'baseline executable bytes mismatch')
    require(sha(m['jadx']) == m['jadx_sha256'], 'JADX executable hash mismatch')
    for lib in m['jadx_libs']:
        require(sha(lib['path']) == lib['sha256'], f'JADX library hash mismatch {lib["path"]}')
    for leg, tools in m['jdk_tools'].items():
        home = Path(tools['home'])
        for tool in ('java', 'javac', 'jar', 'javap'):
            require(sha(home / 'bin' / tool) == tools[f'{tool}_sha256'], f'JDK {leg} {tool} hash mismatch')
    require(sha(CLI_META) == pre['candidate_metadata_sha256'], 'candidate metadata hash mismatch')
    require(sha(HIST) == m['historical_cli8_run_metadata_sha256'], 'CLI8 metadata hash mismatch')
    require(sha(LINKAGE) == EXPECTED_LINKAGE_SHA, 'sibling linkage file hash mismatch')
    require(linkage['field18_not_covered_by_this_replay'] == 18, 'sibling linkage scope changed')
    require(len(linkage['records']) == 18, 'linkage record count mismatch')

    rows = m['cases']
    require(len(rows) == 18, 'expected 18 cases')
    expected_keys = {(family, leg) for family in {
        'RawBoundWriter', 'SCGBCompat', 'ListWrong', 'NullLocal', 'ParamReassigned',
        'ParameterShift', 'RawAllocationVariable', 'RawParamArray', 'StaticRawField'
    } for leg in ('javac8', 'javac23')}
    require({(r['family'], r['leg']) for r in rows} == expected_keys, 'case matrix mismatch')
    old_inputs = {(x['family'], x['leg']): x for x in old['temporary_original_jars']
                  if x.get('kind') == 'source-rebuilt-temp-jar'}
    require(len(old_inputs) == 18, 'historical source-rebuilt input count mismatch')
    link_by_key = {(x['family'], x['leg']): x for x in linkage['records']}
    require(set(link_by_key) == expected_keys, 'linkage matrix mismatch')

    checked_outputs = 0
    for item in m['files']:
        path = OUT / item['path']
        require(path.is_file(), f'missing output file {item["path"]}')
        require(path.stat().st_size == item['bytes'], f'byte count mismatch {item["path"]}')
        require(sha(path) == item['sha256'], f'output hash mismatch {item["path"]}')
        checked_outputs += 1

    commands = m['commands']
    for c in commands:
        require(Path(c['stdout']).is_file() and Path(c['stderr']).is_file(), f'missing stream {c["label"]}')
        require(sha(c['stdout']) == c['stdout_sha256'], f'stdout hash mismatch {c["label"]}')
        require(sha(c['stderr']) == c['stderr_sha256'], f'stderr hash mismatch {c["label"]}')
    require(len(commands) == 274, 'actual command count differs from preserved run')
    command_counts = collections.Counter(c['label'] for c in commands)
    for label in ('original-source-javac', 'permanent-original-jar', 'original-input-runtime',
                  'original-input-javap', 'reference-jadx', 'baseline-class-source',
                  'candidate-class-source', 'original-isolated-javac', 'original-Xverify-runtime',
                  'jadx-isolated-javac', 'baseline-isolated-javac', 'baseline-Xverify-runtime',
                  'candidate-isolated-javac', 'candidate-Xverify-runtime'):
        require(command_counts[label] == 18, f'unexpected count for {label}')
    require(command_counts['jadx-Xverify-runtime'] == 14, 'JADX runtime count mismatch')

    by_label_key = {}
    for c in commands:
        argv = c['argv']
        if c['label'] in ('original-source-javac', 'permanent-original-jar'):
            for row in rows:
                if row['family'] in c['cwd'] and row['leg'] in c['cwd']:
                    by_label_key[(c['label'], row['family'], row['leg'])] = c
        if c['label'].endswith('-isolated-javac'):
            require('-classpath' in argv and argv[argv.index('-classpath') + 1].endswith('/empty-classpath/' + Path(c['cwd']).parts[-2]),
                    f'isolated empty classpath not explicit: {c["label"]}')
            require('-sourcepath' in argv, f'isolated empty sourcepath not explicit: {c["label"]}')
        if c['label'].endswith('-Xverify-runtime') or c['label'] == 'original-input-runtime':
            require('-Xverify:all' in argv, f'Xverify missing: {c["label"]}')
            cp = argv[argv.index('-cp') + 1]
            require('.jar' not in cp, f'input JAR leaked onto runtime classpath: {c["label"]}')

    for r in rows:
        key = (r['family'], r['leg'])
        source = Path(r['source_path'])
        require(source.is_file() and sha(source) == r['source_sha256'] == r['source_copy_sha256'], f'source mismatch {key}')
        h = old_inputs[key]
        require(h['source']['path'] == str(source) and h['source']['sha256'] == r['source_sha256'], f'CLI8 source linkage mismatch {key}')
        require(h['temporary_path'] == r['historical_temp_jar_path'], f'historical JAR path mismatch {key}')
        require(h['jar_sha256'] == r['historical_temp_jar_sha256'], f'historical JAR SHA linkage mismatch {key}')
        require(h['class_sha256'] == r['historical_class_sha256'] == r['rebuilt_class_sha256'], f'class bytes differ from historical class SHA {key}')
        require(r['class_hash_matches_historical'] is True, f'class hash match false {key}')
        require(not Path(r['historical_temp_jar_path']).exists(), f'historical temporary JAR unexpectedly exists {key}')
        require(Path(r['permanent_rebuilt_jar']).is_file(), f'permanent reconstructed JAR missing {key}')
        require(sha(r['permanent_rebuilt_jar']) == r['permanent_rebuilt_jar_sha256'], f'rebuilt JAR SHA mismatch {key}')
        require(r['historical_temp_jar_sha256'] != r['permanent_rebuilt_jar_sha256'], f'JAR byte hashes unexpectedly equal {key}')
        hist_javac = r['historical_original_javac_command']
        hist_jar = r['historical_original_jar_command']
        require(hist_javac in old['actual_commands'] and hist_jar in old['actual_commands'], f'historical command not present in CLI8 record {key}')
        require(sha(hist_javac['stdout']) == hist_javac['stdout_sha256'] and sha(hist_javac['stderr']) == hist_javac['stderr_sha256'], f'historical javac streams mismatch {key}')
        require(sha(hist_jar['stdout']) == hist_jar['stdout_sha256'] and sha(hist_jar['stderr']) == hist_jar['stderr_sha256'], f'historical jar streams mismatch {key}')
        require(hist_javac['exit'] == hist_jar['exit'] == 0, f'historical build did not succeed {key}')
        require(r['original_compile_exit'] == r['original_jar_exit'] == r['original_runtime_exit'] == 0, f'original recovery failed {key}')
        for flavor in ('original', 'baseline', 'candidate'):
            x = r['flavors'][flavor]
            require(x['compile_exit'] == 0 and x['runtime_exit'] == 0 and x['behavior_match'] is True, f'{flavor} behavior failure {key}')
            require(x['classpath'].endswith('/empty-classpath/' + r['leg']), f'{flavor} classpath isolation mismatch {key}')
            require(x['sourcepath'].endswith('/empty-sourcepath/' + r['leg']), f'{flavor} sourcepath isolation mismatch {key}')
        require(r['cli']['baseline']['exit'] == r['cli']['candidate']['exit'] == 0, f'CLI9/baseline source run failed {key}')
        case_cmds = [c for c in commands if f'/{r["leg"]}/{r["family"]}' in c['cwd']]
        for label, flavor in (('baseline-class-source', 'baseline'), ('candidate-class-source', 'candidate')):
            got = [c for c in case_cmds if c['label'] == label]
            require(len(got) == 1, f'{label} command missing/duplicate {key}')
            require(got[0]['exit'] == r['cli'][flavor]['exit'], f'{label} exit mismatch {key}')
            require(got[0]['stdout_sha256'] == r['cli'][flavor]['stdout_sha256'] == r['cli'][flavor]['report_sha256'], f'{label} stdout/report mismatch {key}')
            require(got[0]['stderr_sha256'] == r['cli'][flavor]['stderr_sha256'], f'{label} stderr mismatch {key}')
            require(got[0]['argv'][0] == m['baseline_cli' if flavor == 'baseline' else 'candidate_cli'], f'{label} CLI path mismatch {key}')
        original_run = [c for c in case_cmds if c['label'] == 'original-input-runtime']
        require(len(original_run) == 1, f'original runtime command missing {key}')
        original_bytes = Path(original_run[0]['stdout']).read_bytes()
        semantic = lambda b: [line for line in b.decode(errors='replace').splitlines() if 'GenericType=' not in line and not line.startswith('method=')]
        for flavor in ('original', 'jadx', 'baseline', 'candidate'):
            fd = r['flavors'][flavor]
            if fd['compile_exit'] != 0:
                continue
            label = 'original-Xverify-runtime' if flavor == 'original' else flavor + '-Xverify-runtime'
            runtime = [c for c in case_cmds if c['label'] == label]
            require(len(runtime) == 1 and runtime[0]['exit'] == fd['runtime_exit'], f'{flavor} runtime command mismatch {key}')
            runtime_bytes = Path(runtime[0]['stdout']).read_bytes()
            require(sha(runtime[0]['stdout']) == fd['runtime_stdout_sha256'], f'{flavor} runtime output hash mismatch {key}')
            require((semantic(runtime_bytes) == semantic(original_bytes)) == fd['behavior_match'], f'{flavor} behavior flag mismatch {key}')
            require((runtime_bytes == original_bytes) == fd['api_output_match'], f'{flavor} API output flag mismatch {key}')
        expected_link = link_by_key[key]
        require(expected_link['kind'] == 'source-rebuilt-temp-jar' and expected_link['expected_jar_sha256'] == r['historical_temp_jar_sha256'], f'CLI8-to-CLI9 linkage mismatch {key}')

    behavior_counts = {f: sum(row['flavors'][f].get('behavior_match') is True for row in rows)
                       for f in ('original', 'jadx', 'baseline', 'candidate')}
    api_counts = {f: sum(row['flavors'][f].get('api_output_match') is True for row in rows)
                  for f in ('original', 'jadx', 'baseline', 'candidate')}
    compile_counts = {f: sum(row['flavors'][f]['compile_exit'] == 0 for row in rows)
                      for f in ('original', 'jadx', 'baseline', 'candidate')}
    require(compile_counts == {'original': 18, 'jadx': 14, 'baseline': 18, 'candidate': 18}, 'compile totals changed')
    require(behavior_counts == {'original': 18, 'jadx': 14, 'baseline': 18, 'candidate': 18}, 'behavior totals changed')
    require(api_counts == {'original': 18, 'jadx': 14, 'baseline': 10, 'candidate': 12}, 'API output totals changed')
    report = {
        'status': 'verified', 'scope': '18 field source-rebuilt temporary-JAR inputs only',
        'candidate_cli_sha256': EXPECTED_CANDIDATE_SHA,
        'baseline_cli_sha256': EXPECTED_BASELINE_SHA,
        'source_archive_sha256': EXPECTED_ARCHIVE_SHA,
        'cases': len(rows), 'checked_manifest_files': checked_outputs,
        'checked_actual_commands_and_stream_pairs': len(commands),
        'historical_class_hash_matches': sum(r['class_hash_matches_historical'] for r in rows),
        'historical_jar_byte_hash_matches': sum(r['historical_temp_jar_sha256'] == r['permanent_rebuilt_jar_sha256'] for r in rows),
        'compile_success': compile_counts, 'behavior_match': behavior_counts, 'api_output_match': api_counts,
        'limitations': [
            'Original temporary ZIP JARs were absent; their JAR byte identity is not established. All 18 class hashes match recorded CLI8 class hashes.',
            'JADX isolated compilation failed for ListWrong and ParamReassigned under each JDK (4 cases); preserved stderr is evidence of the failures.'
        ]
    }
    REPORT.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
