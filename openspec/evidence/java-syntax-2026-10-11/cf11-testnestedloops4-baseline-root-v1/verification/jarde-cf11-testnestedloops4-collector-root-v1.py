#!/usr/bin/env python3
"""Deterministic CF11 TestNestedLoops4.testFor collector. Draft: do not run before review."""
from __future__ import annotations
import ast, datetime, hashlib, importlib.util, json, os, re, shutil, subprocess, sys, time, zipfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path('/private/tmp/jarde-cf11-testnestedloops4-baseline-root-v1')
UPSTREAM = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestNestedLoops4.java')
UPSTREAM_SHA256 = 'b84dbf9988e1343f6d26f06345a4e8ba8b17993b4dc33419c86d9e9194968cca'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA256 = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX = Path('/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
JADX_ALIAS = Path('/opt/homebrew/bin/jadx')
JADX_ALIAS_SHA256 = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
JADX_SHA256 = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
JADX_VERSION = '1.5.6'
JADX_CELLAR = Path('/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
JADX_CELLAR_SHA256 = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
JADX_LAUNCHER = Path('/opt/homebrew/Cellar/jadx/1.5.6/libexec/bin/jadx')
JADX_LAUNCHER_SHA256 = '6b732b576bf8dc554278beb94ce006f9549ac898eff975d83548f87244961f13'
JADX_JAR = Path('/opt/homebrew/Cellar/jadx/1.5.6/libexec/lib/jadx-1.5.6-all.jar')
JADX_JAR_SHA256 = '966d314282d95ff7c72f597bb9c744999856666a3b856575e3c035a9c12a0b65'
JADX_RECEIPT = Path('/opt/homebrew/Cellar/jadx/1.5.6/INSTALL_RECEIPT.json')
JADX_RECEIPT_SHA256 = '5c5d3dc861cfc9e61d2ab6da6eece90b9058d354366f769ecbfab1ed2bfc205b'
JADX_REFERENCE_HEAD = '2fb1b16386941660fda07e9017285aec40fcb37f'
CLI = Path('/private/tmp/jarde-computed-for-final-cli-root-v2')
CLI_SHA256 = 'd1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1'
CLI_ACCEPTANCE = Path('/private/tmp/jarde-computed-for-final-cli-acceptance-root-v1.json')
CLI_ACCEPTANCE_SHA256 = '7b6b28e99965a9a8d710ee097d1174c1a60519d2bc2f03b8f690d42082548fea'
JARDE_SCHEMA_SAMPLE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/candidate-replay-root-v3/cases/javac8-jarde-render/class-source-default.json'
JARDE_SCHEMA_SAMPLE_SHA256 = 'd30364f37e15172a1d8ccca6c4371c57005940b55bac65544612adc3081bbbe2'
HELPER = ROOT / 'openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/prepare-baseline-luna-v1.py'
HELPER_SHA256 = 'a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
PACKAGE = 'cf11'
OUTER = 'UpstreamNestedLoops4'
CLI_CLASS = PACKAGE + '.' + OUTER
STRIP_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
TIMEOUT = 60


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict:
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha_bytes(data)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def extract_test_for(source: bytes) -> bytes:
    """Return the exact declaration-through-body bytes, preserving upstream method text."""
    text = source.decode('utf-8')
    match = re.search(r'(?m)^\s*public int testFor\(\)\s*\{', text)
    if not match:
        raise RuntimeError('upstream public int testFor() declaration not found')
    open_brace = text.find('{', match.start())
    depth = 0
    in_string = in_char = escaped = False
    line_comment = block_comment = False
    for index in range(open_brace, len(text)):
        ch = text[index]
        nxt = text[index + 1] if index + 1 < len(text) else ''
        if line_comment:
            if ch == '\n': line_comment = False
            continue
        if block_comment:
            if ch == '*' and nxt == '/': block_comment = False
            continue
        if in_string or in_char:
            if escaped: escaped = False
            elif ch == '\\': escaped = True
            elif in_string and ch == '"': in_string = False
            elif in_char and ch == "'": in_char = False
            continue
        if ch == '/' and nxt == '/': line_comment = True; continue
        if ch == '/' and nxt == '*': block_comment = True; continue
        if ch == '"': in_string = True; continue
        if ch == "'": in_char = True; continue
        if ch == '{': depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0:
                return text[match.start():index + 1].encode('utf-8')
    raise RuntimeError('unbalanced testFor method body')


def adapted_sources(upstream: bytes) -> tuple[bytes, bytes, dict]:
    method = extract_test_for(upstream)
    # This is an adapted single-method fixture, not the unmodified upstream test class.
    source = (f'package {PACKAGE};\npublic class {OUTER} {{\n').encode() + method + b'\n}\n'
    runner = (f'package {PACKAGE};\npublic final class Runner {{\n'
              '    public static void main(String[] args) {\n'
              f'        System.out.println("testFor=" + new {OUTER}().testFor());\n'
              '    }\n}\n').encode()
    offset = upstream.find(method)
    return source, runner, {'upstream_method_sha256': sha_bytes(method),
        'upstream_byte_offset_start': offset, 'upstream_byte_offset_end_exclusive': offset + len(method),
        'method_bytes': len(method), 'method_exact_utf8': method.decode('utf-8'),
        'adapter_note': 'Only upstream TestCls.testFor declaration/body is retained in top-level cf11.UpstreamNestedLoops4. JUnit shell, TestCls owner wrapper and AssertJ are removed. This is adapted source, not the upstream class/classfile.'}


def load_helpers():
    raw = HELPER.read_bytes()
    if sha_bytes(raw) != HELPER_SHA256: raise RuntimeError('pinned helper SHA mismatch')
    tree = ast.parse(raw, filename=str(HELPER))
    wanted = {'b3', 'file_record', 'inventory_rows', 'parse_javap'}
    nodes = [n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef)) and n.name in wanted]
    if {n.name for n in nodes} != wanted: raise RuntimeError('pinned helper API mismatch')
    ns = {'hashlib': hashlib, 'json': json, 'os': os, 're': re, 'shutil': shutil,
          'subprocess': subprocess, 'time': time, 'zipfile': zipfile,
          'datetime': datetime.datetime, 'timezone': datetime.timezone, 'Path': Path,
          'ROOT': ROOT, 'OUT': OUT, 'SOURCE': UPSTREAM, 'RUNNER': UPSTREAM,
          'CLASS_NAME': OUTER, 'STRIPPED_ENV': STRIP_ENV, 'sha': sha_bytes}
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(HELPER), 'exec'), ns)
    return ns


def load_guard_and_scanner():
    if sha_bytes(GUARD.read_bytes()) != GUARD_SHA256 or sha_bytes(SCANNER.read_bytes()) != SCANNER_SHA256:
        raise RuntimeError('v9 guard/scanner pin mismatch')
    sys.dont_write_bytecode = True
    def module_at(name, path):
        spec = importlib.util.spec_from_file_location(name, path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module
    guard = module_at('cf11_validation_guard_v9', GUARD)
    scanner = module_at('cf11_target_size_scanner_v1', SCANNER)
    guard.ROOT = ROOT
    guard.OUT = OUT / 'raw'
    guard.target_bytes = lambda: scanner.target_bytes(ROOT)
    guard.command_stream = lambda path: {'path': str(Path(path).resolve()),
        'bytes': Path(path).stat().st_size, 'sha256': sha_bytes(Path(path).read_bytes())}
    return guard, scanner


class Recorder:
    def __init__(self, helpers, legs, guard):
        self.commands, self.helpers, self.legs, self.guard = [], helpers, legs, guard

    def run(self, label, argv, home=None):
        argv = [str(value) for value in argv]
        env = {key: value for key, value in os.environ.items() if key not in STRIP_ENV}
        if home is not None:
            env['JAVA_HOME'] = str(home)
            env['PATH'] = str(home / 'bin') + os.pathsep + env.get('PATH', '')
        row = self.guard.run_command(len(self.commands) + 1, argv, env)
        row['label'] = label
        row['java_home'] = str(home) if home else None
        self.commands.append(row)
        write_json(OUT / 'execution.json', self.snapshot())
        stdout = Path(row['streams']['stdout']['path']).read_bytes()
        stderr = Path(row['streams']['stderr']['path']).read_bytes()
        return row, stdout, stderr

    def snapshot(self):
        return {'schema': 'cf11-testnestedloops4-luna-v2', 'status': 'running',
                'fixture': 'adapted upstream TestCls.testFor() only; owner is top-level cf11.UpstreamNestedLoops4',
                'commands': self.commands,
                'resource_guard': {'template': str(GUARD), 'template_sha256': GUARD_SHA256,
                    'scanner': str(SCANNER), 'scanner_sha256': SCANNER_SHA256,
                    'minimum_free_bytes': self.guard.FREE_LIMIT,
                    'maximum_target_bytes': self.guard.TARGET_LIMIT,
                    'poll_seconds': 1, 'peak_target_is_sampled': True}}


def class_files(directory: Path):
    return sorted(p for p in directory.rglob('*.class') if p.is_file())


def compile_and_run(rec, helpers, label, sources, leg, expected_paths, oracle=None):
    case = OUT / 'legs' / label; classes = case / 'classes'; empty = case / 'empty-paths'
    classes.mkdir(parents=True, exist_ok=False); empty.mkdir()
    javac, java, home = leg['tools']['javac'], leg['tools']['java'], leg['home']
    release_flags = (['-source', '8', '-target', '8'] if leg['leg'] == 'javac8' else ['--release', '8'])
    comp, _, _ = rec.run(label + '-compile', [javac, *release_flags, '-proc:none', '-g', '-Xlint:-options',
        '-classpath', empty, '-sourcepath', empty, '-d', classes, *sources], home)
    run = None
    if comp['exit_code'] == 0 and comp['guard_stop'] is None:
        run, _, _ = rec.run(label + '-run', [java, '-Xverify:all', '-cp', classes, PACKAGE + '.Runner'], home)
    outputs = class_files(classes)
    actual = sorted(p.relative_to(classes).as_posix() for p in outputs)
    runtime_match = None
    if run is not None and oracle is not None:
        actual_stdout = Path(run['streams']['stdout']['path']).read_bytes()
        actual_stderr = Path(run['streams']['stderr']['path']).read_bytes()
        runtime_match = (run['exit_code'], actual_stdout, actual_stderr) == (oracle['exit_code'], oracle['stdout'], oracle['stderr'])
    return {'label': label, 'compile_command': comp['label'], 'runtime_command': run['label'] if run else None,
        'source_pins': [helpers['file_record'](Path(s)) for s in sources],
        'classes': [helpers['file_record'](p) for p in outputs], 'class_paths': actual,
        'expected_class_paths': expected_paths, 'class_set_exact': actual == expected_paths,
        'compile_exit': comp['exit_code'], 'compile_guard': comp['guard_stop'],
        'run_exit': run['exit_code'] if run else None, 'run_guard': run['guard_stop'] if run else None,
        'same_jdk_original_raw': runtime_match, 'status': 'observed', 'acceptance': None}


def main():
    if OUT.exists(): raise SystemExit(f'refuse overwrite: {OUT}')
    OUT.mkdir(parents=True); (OUT / 'legs').mkdir(); (OUT / 'raw').mkdir(); (OUT / 'inputs').mkdir()
    helpers = load_helpers()
    guard, scanner = load_guard_and_scanner()
    (OUT / 'raw').mkdir(parents=True, exist_ok=True)
    upstream = UPSTREAM.read_bytes()
    if sha_bytes(upstream) != UPSTREAM_SHA256: raise SystemExit('upstream source pin mismatch; no commands run')
    adapted, runner, extraction = adapted_sources(upstream)
    (OUT / 'inputs' / (OUTER + '.java')).write_bytes(adapted)
    (OUT / 'inputs' / 'Runner.java').write_bytes(runner)
    write_json(OUT / 'adaptation.json', {**extraction, 'upstream_pin': pin(UPSTREAM),
        'adapted_source_pin': pin(OUT / 'inputs' / (OUTER + '.java')),
        'runner_pin': pin(OUT / 'inputs' / 'Runner.java'), 'target_method': 'adapted cf11.UpstreamNestedLoops4.testFor()I; upstream owner was nested TestCls',
        'expected_observation': 'testFor=12', 'expected_observation_is_oracle_not_result': True})
    preflight = []
    for label, path, expected in [('upstream', UPSTREAM, UPSTREAM_SHA256), ('JADX-Cellar-runtime-command', JADX, JADX_SHA256),
                                  ('JADX-homebrew-alias', JADX_ALIAS, JADX_ALIAS_SHA256),
                                  ('JADX-cellar-wrapper', JADX_CELLAR, JADX_CELLAR_SHA256),
                                  ('JADX-runtime-launcher', JADX_LAUNCHER, JADX_LAUNCHER_SHA256),
                                  ('JADX-runtime-jar', JADX_JAR, JADX_JAR_SHA256),
                                  ('JADX-install-receipt', JADX_RECEIPT, JADX_RECEIPT_SHA256),
                                  ('Jarde CLI', CLI, CLI_SHA256), ('CLI acceptance receipt', CLI_ACCEPTANCE, CLI_ACCEPTANCE_SHA256),
                                  ('Jarde class-source schema sample', JARDE_SCHEMA_SAMPLE, JARDE_SCHEMA_SAMPLE_SHA256),
                                  ('JDK manifest', JDK_MANIFEST, JDK_MANIFEST_SHA256), ('helper', HELPER, HELPER_SHA256)]:
        actual = sha_bytes(path.read_bytes()) if path.is_file() else None
        preflight.append({'label': label, 'path': str(path), 'expected_sha256': expected, 'actual_sha256': actual,
                          'ok': actual == expected})
    jdk_doc = json.loads(JDK_MANIFEST.read_bytes())
    legs = {}
    for record in jdk_doc.get('legs', []):
        tool_rows = {}
        for name, item in record.get('jdk_tools', {}).items():
            path = Path(item['path']); actual = sha_bytes(path.read_bytes()) if path.is_file() else None
            tool_rows[name] = {'path': str(path), 'expected_sha256': item['sha256'], 'actual_sha256': actual,
                               'ok': actual == item['sha256']}
        if {'java', 'javac', 'javap'} <= tool_rows.keys():
            legs[record['leg']] = {'tools': {k: Path(v['path']) for k, v in tool_rows.items()},
                                   'home': Path(tool_rows['java']['path']).parent.parent, 'tool_pins': tool_rows, 'leg': record['leg']}
    receipt = json.loads(CLI_ACCEPTANCE.read_bytes())
    schema_sample = json.loads(JARDE_SCHEMA_SAMPLE.read_bytes())
    schema_sample_ok = (isinstance(schema_sample, dict) and isinstance(schema_sample.get('text'), str)
                        and isinstance(schema_sample.get('methods'), list)
                        and isinstance(schema_sample.get('execution'), dict)
                        and schema_sample['execution'].get('status') == 'complete')
    preflight.append({'label': 'jarde-class-source-json-schema', 'ok': schema_sample_ok,
                      'sample_path': str(JARDE_SCHEMA_SAMPLE), 'sample_sha256': JARDE_SCHEMA_SAMPLE_SHA256,
                      'top_level_keys': sorted(schema_sample) if isinstance(schema_sample, dict) else None,
                      'text_type': type(schema_sample.get('text')).__name__ if isinstance(schema_sample, dict) else None,
                      'methods_type': type(schema_sample.get('methods')).__name__ if isinstance(schema_sample, dict) else None,
                      'execution_status': schema_sample.get('execution', {}).get('status') if isinstance(schema_sample, dict) else None})
    final_cli = receipt.get('final_cli', {})
    cli_meta_ok = (receipt.get('status') == 'accepted' and receipt.get('source_input_count') == 4700
                   and receipt.get('source_pins_unchanged') is True and final_cli.get('path') == str(CLI)
                   and final_cli.get('sha256') == CLI_SHA256 and final_cli.get('mode') == '0o555')
    preflight.append({'label': 'cli-acceptance-link', 'ok': cli_meta_ok,
                      'receipt_path': str(CLI_ACCEPTANCE), 'final_cli': final_cli,
                      'build_sha256': receipt.get('build_sha256')})
    preflight.append({'label': 'jdk-leg-set', 'expected': ['javac8', 'javac23'],
                      'actual': sorted(legs), 'ok': set(legs) == {'javac8', 'javac23'}})
    for leg_name, leg in legs.items():
        preflight.extend({'label': leg_name + ':' + name, **facts} for name, facts in leg['tool_pins'].items())
    write_json(OUT / 'preflight.json', preflight)
    if any(not p['ok'] for p in preflight):
        write_json(OUT / 'execution.json', {'schema': 'cf11-testnestedloops4-luna-v2',
            'status': 'preflight-failed', 'preflight': preflight, 'commands': []}); return 1

    rec = Recorder(helpers, legs, guard)
    source_path = OUT / 'inputs' / (OUTER + '.java'); runner_path = OUT / 'inputs' / 'Runner.java'
    expected = sorted([f'{PACKAGE}/{OUTER}.class', f'{PACKAGE}/Runner.class'])
    pre_run_pin_paths = [Path(__file__), UPSTREAM, source_path, runner_path, HELPER, GUARD, SCANNER, JDK_MANIFEST, CLI,
                         CLI_ACCEPTANCE, JARDE_SCHEMA_SAMPLE, JADX, JADX_ALIAS, JADX_CELLAR, JADX_LAUNCHER, JADX_JAR, JADX_RECEIPT]
    pre_run_pin_paths += [Path(item['path']) for leg in legs.values() for item in leg['tool_pins'].values()]
    pre_run_pins = [pin(path) for path in pre_run_pin_paths]
    rows, oracle = [], {}
    original_classes = {}
    for leg_name in ('javac8', 'javac23'):
        leg = legs[leg_name]
        row = compile_and_run(rec, helpers, leg_name + '-original', [source_path, runner_path], leg, expected)
        target = OUT / 'legs' / (leg_name + '-original') / 'classes' / PACKAGE / (OUTER + '.class')
        row['kind'] = 'adapted-original'; row['input_sha256'] = sha_bytes(adapted)
        rows.append(row)
        if target.exists(): original_classes[leg_name] = target
        if target.exists():
            javap_label = leg_name + '-original-javap'
            jp, jout, jerr = rec.run(javap_label, [leg['tools']['javap'], '-p', '-c', '-s', '-v',
                '-classpath', OUT / 'legs' / (leg_name + '-original') / 'classes', PACKAGE + '.' + OUTER], leg['home'])
            javap_path = OUT / 'legs' / (leg_name + '-original') / 'javap.txt'
            javap_path.write_bytes(jout)
            try:
                physical = helpers['parse_javap'](jout.decode('utf-8', errors='replace'))
                method_rows = physical.get('methods', {})
                fact = method_rows.get(('testFor', '()I'))
                test_facts = ({'flags': fact['flags'], 'bcis': list(fact['bcis']),
                               'declaration': fact['declaration']} if fact is not None else None)
                if test_facts is not None and not all(isinstance(k, str) for k in test_facts):
                    raise TypeError('javap facts are not JSON-safe')
            except Exception as exc:
                test_facts = None
                row['javap_parse_error'] = f'{type(exc).__name__}: {exc}'
            row['physical_method'] = {'javap_command': javap_label, 'javap_pin': pin(javap_path),
                'testFor_descriptor': '()I', 'testFor_facts': test_facts,
                'contains_return_zero_source': b'return 0;' in adapted,
                'return_zero_is_runtime_reachable': 'not assumed; original testFor has no input and check()==12 does not establish that path'}
        run_label = row['runtime_command']
        if run_label:
            command = next(c for c in rec.commands if c['label'] == run_label)
            oracle[leg_name] = {'exit_code': command['exit_code'],
                'stdout': Path(command['streams']['stdout']['path']).read_bytes(),
                'stderr': Path(command['streams']['stderr']['path']).read_bytes()}

    input_class = original_classes.get('javac23')
    jar = OUT / 'jadx-input' / 'UpstreamNestedLoops4.jar'; jar.parent.mkdir()
    jar_exact = False
    if input_class:
        payload = input_class.read_bytes()
        with zipfile.ZipFile(jar, 'w', zipfile.ZIP_STORED) as zf:
            info = zipfile.ZipInfo(f'{PACKAGE}/{OUTER}.class', date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            zf.writestr(info, payload)
        with zipfile.ZipFile(jar) as zf:
            jar_exact = zf.namelist() == [f'{PACKAGE}/{OUTER}.class'] and zf.read(f'{PACKAGE}/{OUTER}.class') == payload
    version, version_out, _ = rec.run('jadx-version', [JADX, '--version'], legs['javac23']['home'])
    jadx_rows = {}
    for profile in ('default', 'none'):
        outdir = OUT / 'jadx' / profile
        outdir.parent.mkdir(parents=True, exist_ok=True)
        argv = [JADX, '--no-res', '--config', 'none', '--threads-count', '1']
        if profile == 'none': argv += ['--rename-flags', 'none']
        decomp, _, _ = rec.run('jadx-' + profile, [*argv, '-d', outdir, jar], legs['javac23']['home'])
        sources = sorted(outdir.rglob('*.java')) if outdir.exists() else []
        packages = []
        for p in sources:
            m = re.search(r'(?m)^\s*package\s+([\w.]+)\s*;', p.read_text(encoding='utf-8'))
            packages.append(m.group(1) if m else None)
        jadx_rows[profile] = {'command': decomp['label'], 'profile': profile,
            'generated_sources': [helpers['file_record'](p) for p in sources],
            'source_names': [p.name for p in sources], 'packages': packages,
            'exact_expected_names': [p.name for p in sources] == [OUTER + '.java'],
            'input_jar': pin(jar) if jar.exists() else None, 'input_jar_exact': jar_exact,
            'expected_package': PACKAGE, 'package_match': packages == [PACKAGE],
            'exit_code': decomp['exit_code'], 'guard_stop': decomp['guard_stop']}
        if len(sources) == 1 and packages == [PACKAGE]:
            source_copy = OUT / 'inputs' / f'jadx-{profile}' / (OUTER + '.java')
            source_copy.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(sources[0], source_copy)
            for leg_name in ('javac8', 'javac23'):
                rows.append(compile_and_run(rec, helpers, f'{leg_name}-jadx-{profile}', [source_copy, runner_path],
                                            legs[leg_name], expected, oracle.get(leg_name))
                            | {'kind': 'jadx', 'profile': profile})
        else:
            rows.append({'kind': 'jadx', 'profile': profile, 'blocked': 'generated source/package set is not exactly the expected one; no compile assumed'})

    jarde_rows = []
    for leg_name in ('javac8', 'javac23'):
        original = original_classes.get(leg_name)
        for mode in ('default', 'all'):
            label = f'{leg_name}-jarde-{mode}'
            if original is None:
                jarde_rows.append({'label': label, 'render': None, 'compile_run': None, 'status': 'blocked-no-input-class'}); continue
            argv = [CLI, 'class-source', '--input', original, '--class', CLI_CLASS,
                    '--policy', 'single-class', '--release', '8', '--format', 'json']
            if mode == 'all': argv += ['--evidence', 'all']
            render, stdout, stderr = rec.run(label + '-render', argv, legs[leg_name]['home'])
            row = {'label': label, 'mode': mode, 'render_command': render['label'],
                   'render_exit': render['exit_code'], 'render_guard': render['guard_stop'],
                   'report_raw': render['streams'], 'compile_run': None, 'class_set_exact': None}
            if render['exit_code'] == 0 and render['guard_stop'] is None:
                try:
                    report = json.loads(stdout)
                    if not isinstance(report, dict) or not isinstance(report.get('text'), str):
                        raise TypeError('class-source JSON report lacks string text field')
                    doc_path = OUT / 'jarde' / label / 'report.json'; write_json(doc_path, report)
                    text = report['text']
                    src = OUT / 'jarde' / label / (OUTER + '.java'); src.parent.mkdir(parents=True, exist_ok=True)
                    src.write_text(text, encoding='utf-8')
                    compiled = compile_and_run(rec, helpers, label, [src, runner_path], legs[leg_name], expected, oracle.get(leg_name))
                    row.update({'document': pin(doc_path), 'rendered_text': pin(src), 'rendered_text_path': str(src),
                                'report_status': report.get('execution', {}).get('status'),
                                'complete_report_json': True, 'text_field_type': type(report.get('text')).__name__, 'compile_run': compiled,
                                'class_set_exact': compiled['class_set_exact'],
                                'same_jdk_runtime_oracle': compiled['same_jdk_original_raw']})
                except (KeyError, TypeError, ValueError, UnicodeError) as exc:
                    row['report_parse_error'] = f'{type(exc).__name__}: {exc}'
            jarde_rows.append(row)
    for leg_name in ('javac8', 'javac23'):
        pair = [r for r in jarde_rows if r.get('label') in {f'{leg_name}-jarde-default', f'{leg_name}-jarde-all'}]
        if len(pair) == 2 and all(r.get('rendered_text_path') for r in pair):
            pair_texts = [Path(r['rendered_text_path']).read_bytes() for r in pair]
            for r in pair: r['default_all_text_equal'] = pair_texts[0] == pair_texts[1]
    # Profile/source comparisons are observations; no result is predeclared as success.
    tracked = [Path(__file__), UPSTREAM, source_path, runner_path, HELPER, GUARD, SCANNER, JDK_MANIFEST, CLI, CLI_ACCEPTANCE, JARDE_SCHEMA_SAMPLE,
               JADX, JADX_ALIAS, JADX_CELLAR, JADX_LAUNCHER, JADX_JAR, JADX_RECEIPT]
    tracked += [Path(item['path']) for leg in legs.values() for item in leg['tool_pins'].values()]
    post_pins = [pin(path) for path in tracked if path.is_file()]
    pre_hash = {p['path']: p['sha256'] for p in pre_run_pins}
    record = rec.snapshot(); record.update({'status': 'observed', 'runner': pin(Path(__file__)), 'preflight': preflight,
        'post_run_tracked_pins': post_pins, 'source_tool_input_pins_unchanged': None,
        'upstream': pin(UPSTREAM), 'adapted_fixture': pin(source_path), 'runner': pin(runner_path),
        'adaptation': extraction, 'target_method': 'adapted cf11.UpstreamNestedLoops4.testFor()I; upstream owner was nested TestCls',
        'original_runtime_oracles': {k: {'exit_code': v['exit_code'],
             'stdout_sha256': sha_bytes(v['stdout']), 'stderr_sha256': sha_bytes(v['stderr'])} for k,v in oracle.items()},
        'javac_jadx_legs': rows, 'jadx_profiles': jadx_rows, 'jarde_profiles': jarde_rows,
        'jadx_identity': {'upstream_reference_head': JADX_REFERENCE_HEAD,
            'runtime_version_command': version, 'runtime_version_exit': version['exit_code'],
            'runtime_version_stdout_sha256': sha_bytes(version_out),
            'runtime_version_stdout': version_out.decode('utf-8', errors='replace').strip(),
            'expected_runtime_version': JADX_VERSION,
            'runtime_is_existing_homebrew_install': str(JADX_CELLAR),
            'not_built_or_installed_by_collector': True},
        'compile_profile': {'debug_info': True, 'release': 8, 'no_debug_variant': 'not run; debug baseline only'},
        'cli': {'path': str(CLI), 'sha256': CLI_SHA256, 'acceptance_receipt': pin(CLI_ACCEPTANCE)},
        'tool_pins': {k: {name: facts for name,facts in leg['tool_pins'].items()} for k,leg in legs.items()},
        'comparison_policy': 'Record outcomes only. Runtime expectation is sourced from adapted-original per same JDK; no compile/render/runtime success is preset.'})
    record['pre_run_tracked_pins'] = pre_run_pins
    record['source_tool_input_pins_unchanged'] = (len(post_pins) == len(pre_run_pins)
        and all(p['sha256'] == pre_hash.get(p['path']) for p in post_pins))
    write_json(OUT / 'execution.json', record)
    # Pin the complete created inventory after all records are written; exclude this manifest itself.
    files = sorted(p for p in OUT.rglob('*') if p.is_file() and p.name != 'file-inventory.json')
    write_json(OUT / 'file-inventory.json', [helpers['file_record'](p) for p in files])
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
