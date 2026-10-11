#!/usr/bin/env python3
"""Root-run collector draft for CF11 nested-loop side-exit controls; do not run here."""
from __future__ import annotations
import datetime, hashlib, importlib.util, json, os, shutil, sys, zipfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path('/private/tmp/jarde-cf11-side-exits-controls-baseline-root-v1')
BASELINE = Path('/private/tmp/jarde-cf11-testnestedloops4-collector-root-v1.py')
BASELINE_SHA256 = 'e247baa19fd9b7c1d84e451737ed7611cb4c973f1e31c174280cf21f6c32e879'
SOURCE = Path(__file__).with_name('NestedLoopSideExitControls.java')
RUNNER = Path(__file__).with_name('Runner.java')
PACKAGE = 'cf11'
CLASS = 'NestedLoopSideExitControls'
CLI = Path('/private/tmp/jarde-computed-for-final-cli-root-v2')
CLI_SHA256 = 'd1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1'
CLI_ACCEPTANCE = Path('/private/tmp/jarde-computed-for-final-cli-acceptance-root-v1.json')
CLI_ACCEPTANCE_SHA256 = '7b6b28e99965a9a8d710ee097d1174c1a60519d2bc2f03b8f690d42082548fea'
STRIP_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict:
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    if OUT.exists():
        raise SystemExit(f'refuse overwrite: {OUT}')
    if sha(BASELINE.read_bytes()) != BASELINE_SHA256:
        raise SystemExit('accepted baseline collector pin mismatch; no commands run')
    base = load_module('cf11_side_exit_baseline', BASELINE)
    expected_tools = [(base.JDK_MANIFEST, base.JDK_MANIFEST_SHA256),
        (base.JADX, base.JADX_SHA256), (base.JADX_ALIAS, base.JADX_ALIAS_SHA256),
        (base.JADX_CELLAR, base.JADX_CELLAR_SHA256),
        (base.JADX_LAUNCHER, base.JADX_LAUNCHER_SHA256),
        (base.JADX_JAR, base.JADX_JAR_SHA256),
        (base.JADX_RECEIPT, base.JADX_RECEIPT_SHA256)]
    for path, expected in expected_tools:
        if sha(path.read_bytes()) != expected:
            raise SystemExit('fixed tool pin mismatch: ' + str(path))
    # Reuse the accepted CF11 source/tool/guard mechanism and its frozen toolchain pins.
    base.OUT = OUT
    base.PACKAGE = PACKAGE
    base.OUTER = CLASS
    base.CLI_CLASS = PACKAGE + '.' + CLASS
    helpers = base.load_helpers()
    guard, scanner = base.load_guard_and_scanner()
    manifest = json.loads(base.JDK_MANIFEST.read_bytes())
    legs = {}
    for record in manifest.get('legs', []):
        tools = record.get('jdk_tools', {})
        if not {'java', 'javac', 'javap'} <= tools.keys():
            continue
        pins = {}
        for name, item in tools.items():
            path = Path(item['path'])
            actual = sha(path.read_bytes()) if path.is_file() else None
            pins[name] = {'path': str(path), 'expected_sha256': item['sha256'],
                          'actual_sha256': actual, 'ok': actual == item['sha256']}
        if all(row['ok'] for row in pins.values()):
            legs[record['leg']] = {'leg': record['leg'], 'home': Path(pins['java']['path']).parent.parent,
                                   'tools': {name: Path(row['path']) for name, row in pins.items()},
                                   'tool_pins': pins}
    if set(legs) != {'javac8', 'javac23'}:
        raise SystemExit('expected pinned javac8/javac23 legs unavailable; no commands run')
    if sha(CLI.read_bytes()) != CLI_SHA256 or (CLI.stat().st_mode & 0o777) != 0o555:
        raise SystemExit('frozen candidate CLI pin/mode mismatch; no commands run')
    if sha(CLI_ACCEPTANCE.read_bytes()) != CLI_ACCEPTANCE_SHA256:
        raise SystemExit('CLI acceptance receipt pin mismatch; no commands run')
    receipt = json.loads(CLI_ACCEPTANCE.read_bytes())
    final_cli = receipt.get('final_cli', {})
    if (receipt.get('status') != 'accepted' or final_cli.get('path') != str(CLI)
            or final_cli.get('sha256') != CLI_SHA256 or final_cli.get('mode') != '0o555'):
        raise SystemExit('CLI acceptance receipt does not bind the pinned frozen executable; no commands run')

    OUT.mkdir(parents=True)
    for name in ('raw', 'legs', 'inputs', 'jadx', 'jarde'):
        (OUT / name).mkdir()
    shutil.copyfile(SOURCE, OUT / 'inputs' / SOURCE.name)
    shutil.copyfile(RUNNER, OUT / 'inputs' / RUNNER.name)
    source, runner = OUT / 'inputs' / SOURCE.name, OUT / 'inputs' / RUNNER.name

    class Recorder:
        def __init__(self): self.commands = []
        def snapshot(self):
            return {'schema': 'cf11-nested-loop-side-exits-controls-root-v1', 'status': 'running',
                    'commands': self.commands,
                    'resource_guard': {'template': str(base.GUARD), 'template_sha256': base.GUARD_SHA256,
                        'scanner': str(base.SCANNER), 'scanner_sha256': base.SCANNER_SHA256,
                        'minimum_free_bytes': guard.FREE_LIMIT, 'maximum_target_bytes': guard.TARGET_LIMIT,
                        'poll_seconds': 1, 'peak_target_is_sampled': True}}
        def run(self, label, argv, home=None):
            env = {k: v for k, v in os.environ.items() if k not in STRIP_ENV}
            if home is not None:
                env['JAVA_HOME'] = str(home)
                env['PATH'] = str(home / 'bin') + os.pathsep + env.get('PATH', '')
            row = guard.run_command(len(self.commands) + 1, [str(x) for x in argv], env)
            row['label'] = label
            row['java_home'] = str(home) if home else None
            self.commands.append(row)
            write_json(OUT / 'execution.json', self.snapshot())
            if row.get('guard_stop') is not None:
                raise RuntimeError(f"resource guard stopped at {label}: {row['guard_stop']}")
            return row, Path(row['streams']['stdout']['path']).read_bytes(), Path(row['streams']['stderr']['path']).read_bytes()

    rec = Recorder()
    tracked = [Path(__file__), BASELINE, SOURCE, RUNNER, CLI, CLI_ACCEPTANCE, base.JDK_MANIFEST, base.HELPER,
        base.GUARD, base.SCANNER, base.UPSTREAM, base.JADX, base.JADX_ALIAS, base.JADX_CELLAR,
        base.JADX_LAUNCHER, base.JADX_JAR, base.JADX_RECEIPT]
    tracked += [Path(item['path']) for leg in legs.values() for item in leg['tool_pins'].values()]
    pre_pins = [pin(p) for p in tracked if p.is_file()]
    pre_hashes = {row['path']: row['sha256'] for row in pre_pins}
    expected_classes = sorted([f'{PACKAGE}/{CLASS}.class', f'{PACKAGE}/Runner.class'])

    def compile_run(label, source_paths, leg, oracle=None):
        case = OUT / 'legs' / label
        classes, empty = case / 'classes', case / 'empty'
        classes.mkdir(parents=True); empty.mkdir()
        flags = ['-source', '8', '-target', '8'] if leg['leg'] == 'javac8' else ['--release', '8']
        comp, _, _ = rec.run(label + '-compile', [leg['tools']['javac'], *flags, '-proc:none', '-g',
            '-Xlint:-options', '-classpath', empty, '-sourcepath', empty, '-d', classes, *source_paths], leg['home'])
        run = None
        if comp['exit_code'] == 0:
            run, stdout, stderr = rec.run(label + '-run', [leg['tools']['java'], '-Xverify:all', '-cp', classes,
                PACKAGE + '.Runner'], leg['home'])
        else:
            stdout = stderr = None
        files = sorted(p for p in classes.rglob('*.class') if p.is_file())
        actual_paths = sorted(p.relative_to(classes).as_posix() for p in files)
        return {'label': label, 'compile': comp['label'], 'compile_exit': comp['exit_code'],
            'run': run['label'] if run else None, 'run_exit': run['exit_code'] if run else None,
            'run_stdout_sha256': sha(stdout) if stdout is not None else None,
            'run_stdout': stdout.decode('utf-8', errors='replace') if stdout is not None else None,
            'run_stderr_sha256': sha(stderr) if stderr is not None else None,
            'same_jdk_original_stdout': (stdout == oracle['stdout'] and stderr == oracle['stderr']) if run and oracle else None,
            'source_pins': [pin(Path(p)) for p in source_paths],
            'class_paths': actual_paths, 'class_set_exact': actual_paths == expected_classes,
            'classes': [pin(p) for p in files]}

    oracle, original, generated = {}, {}, {}
    rows = []
    for leg_name in ('javac8', 'javac23'):
        leg = legs[leg_name]
        row = compile_run(leg_name + '-original', [source, runner], leg)
        rows.append(row)
        cls = OUT / 'legs' / (leg_name + '-original') / 'classes' / PACKAGE / (CLASS + '.class')
        if cls.is_file(): original[leg_name] = cls
        if row['run']:
            command = next(x for x in rec.commands if x['label'] == row['run'])
            oracle[leg_name] = {'stdout': Path(command['streams']['stdout']['path']).read_bytes(),
                                'stderr': Path(command['streams']['stderr']['path']).read_bytes(),
                                'exit_code': command['exit_code']}
        if cls.is_file():
            jp, jout, jerr = rec.run(leg_name + '-javap', [leg['tools']['javap'], '-p', '-c', '-s', '-v',
                '-classpath', cls.parent.parent, PACKAGE + '.' + CLASS], leg['home'])
            (OUT / 'legs' / (leg_name + '-original') / 'javap.txt').write_bytes(jout)
            rows[-1]['javap_command'] = jp['label']
            rows[-1]['javap_stdout_sha256'] = sha(jout)
            rows[-1]['javap_stderr_sha256'] = sha(jerr)

    if 'javac23' in original:
        jar = OUT / 'inputs' / (CLASS + '.jar')
        payload = original['javac23'].read_bytes()
        with zipfile.ZipFile(jar, 'w', zipfile.ZIP_STORED) as zf:
            info = zipfile.ZipInfo(f'{PACKAGE}/{CLASS}.class', date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            zf.writestr(info, payload)
        with zipfile.ZipFile(jar) as zf:
            jar_exact = zf.namelist() == [f'{PACKAGE}/{CLASS}.class'] and zf.read(f'{PACKAGE}/{CLASS}.class') == payload
        version, version_out, _ = rec.run('jadx-version', [base.JADX, '--version'], legs['javac23']['home'])
        profiles = {}
        for profile in ('default', 'none'):
            outdir = OUT / 'jadx' / profile
            argv = [base.JADX, '--no-res', '--config', 'none', '--threads-count', '1']
            if profile == 'none': argv += ['--rename-flags', 'none']
            dec, _, _ = rec.run('jadx-' + profile, [*argv, '-d', outdir, jar], legs['javac23']['home'])
            sources = sorted(outdir.rglob('*.java'))
            package_names = []
            for p in sources:
                import re
                m = re.search(r'(?m)^\s*package\s+([\w.]+)\s*;', p.read_text(encoding='utf-8'))
                package_names.append(m.group(1) if m else None)
            profile_row = {'command': dec['label'], 'exit_code': dec['exit_code'], 'guard_stop': dec['guard_stop'],
                'input_jar': pin(jar), 'input_jar_exact': jar_exact,
                'sources': [pin(p) for p in sources], 'source_names': [p.name for p in sources],
                'packages': package_names, 'generated_class_set_exact': [p.name for p in sources] == [CLASS + '.java']
                    and package_names == [PACKAGE]}
            profiles[profile] = profile_row
            if profile_row['generated_class_set_exact']:
                copied = OUT / 'inputs' / f'jadx-{profile}' / (CLASS + '.java')
                copied.parent.mkdir(parents=True); shutil.copyfile(sources[0], copied)
                for leg_name in ('javac8', 'javac23'):
                    rows.append(compile_run(f'{leg_name}-jadx-{profile}', [copied, runner], legs[leg_name], oracle.get(leg_name)))
            else:
                rows.append({'label': 'jadx-' + profile, 'blocked': 'generated class/package inventory differs; no compile assumed'})
        jadx_identity = {'version_command': version['label'], 'version_exit': version['exit_code'],
                         'version_stdout': version_out.decode('utf-8', errors='replace').strip(), 'profiles': profiles}
    else:
        jadx_identity = {'blocked': 'javac23 original class absent'}

    jarde = []
    jarde_reports = {}
    for leg_name in ('javac8', 'javac23'):
        cls = original.get(leg_name)
        if cls is None:
            jarde.append({'leg': leg_name, 'blocked': 'no original class'}); continue
        for mode in ('default', 'all'):
            label = f'{leg_name}-jarde-{mode}'
            argv = [CLI, 'class-source', '--input', cls, '--class', PACKAGE + '.' + CLASS,
                    '--policy', 'single-class', '--release', '8', '--format', 'json']
            if mode == 'all': argv += ['--evidence', 'all']
            render, stdout, stderr = rec.run(label + '-render', argv, legs[leg_name]['home'])
            item = {'label': label, 'mode': mode, 'render_command': render['label'], 'render_exit': render['exit_code'],
                    'render_stdout_sha256': sha(stdout), 'render_stderr_sha256': sha(stderr),
                    'render_stdout_path': render['streams']['stdout']['path'], 'compile_run': None}
            if render['exit_code'] == 0:
                try:
                    report = json.loads(stdout)
                    doc = OUT / 'jarde' / label / 'report.json'; write_json(doc, report)
                    text = report['text']
                    src = OUT / 'jarde' / label / (CLASS + '.java'); src.parent.mkdir(parents=True); src.write_text(text, encoding='utf-8')
                    compiled = compile_run(label, [src, runner], legs[leg_name], oracle.get(leg_name))
                    item.update({'report_pin': pin(doc), 'rendered_source': pin(src), 'report_status': report.get('execution', {}).get('status'),
                                 'method_records': report.get('methods'), 'compile_run': compiled})
                    method_maps = []
                    for method in report.get('methods', []):
                        body = method.get('outcome', {}).get('report')
                        method_maps.append({'item': method.get('item'), 'text': method.get('text'),
                            'body': body.get('text') if isinstance(body, dict) else None,
                            'source_map': body.get('source_map') if isinstance(body, dict) else None})
                    jarde_reports[(leg_name, mode)] = {'text': text.encode('utf-8'), 'methods': method_maps}
                except (ValueError, KeyError, TypeError, UnicodeError) as exc:
                    item['parse_error'] = f'{type(exc).__name__}: {exc}'
            jarde.append(item)
    for leg_name in ('javac8', 'javac23'):
        # Compare reports only when both profiles produced parseable complete JSON.
        reports = [jarde_reports.get((leg_name, mode)) for mode in ('default', 'all')]
        if all(reports):
            equal = (reports[0]['text'] == reports[1]['text'] and reports[0]['methods'] == reports[1]['methods'])
            for row in jarde:
                if row['label'].startswith(leg_name + '-jarde-'):
                    row['default_all_text_identity_and_maps_equal'] = equal

    post_pins = [pin(p) for p in tracked if p.is_file()]
    pins_unchanged = (len(post_pins) == len(pre_pins) and all(pre_hashes.get(row['path']) == row['sha256'] for row in post_pins))
    record = rec.snapshot()
    record.update({'status': 'observed', 'fixture_source_pin': pin(SOURCE), 'runner_pin': pin(RUNNER),
        'input_copies': [pin(source), pin(runner)], 'cli': pin(CLI), 'cli_acceptance': pin(CLI_ACCEPTANCE),
        'tool_manifest': pin(base.JDK_MANIFEST), 'tool_pins': {name: leg['tool_pins'] for name, leg in legs.items()},
        'original_runtime_oracles': {k: {'exit_code': v['exit_code'], 'stdout': v['stdout'].decode('utf-8', errors='replace'),
            'stdout_sha256': sha(v['stdout']), 'stderr_sha256': sha(v['stderr'])} for k, v in oracle.items()},
        'compile_and_runtime_rows': rows, 'jadx': jadx_identity, 'jarde': jarde,
        'expected_observations_are_predictions_only': True,
        'scope': 'Full adapted controls class, no handler; exact original per-JDK runtime is the oracle. No outcome is predeclared.'})
    record['tracked_pins_before_after'] = {'before': pre_pins, 'after': post_pins,
        'inputs_and_tools_unchanged': pins_unchanged}
    write_json(OUT / 'execution.json', record)
    inventory = sorted(p for p in OUT.rglob('*') if p.is_file() and p.name != 'file-inventory.json')
    write_json(OUT / 'file-inventory.json', [helpers['file_record'](p) for p in inventory])
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
