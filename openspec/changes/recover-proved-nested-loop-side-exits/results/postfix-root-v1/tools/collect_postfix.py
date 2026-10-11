#!/usr/bin/env python3
"""Post-fix CF11 full-source collector template. Do not run until root freezes inputs."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
import os
import shutil
import sys
import zipfile
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
STRIP_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
CASES = [(-1,54,4),(12,54,4),(13,54,4),(100,54,4),(12,0,4),(12,8,4),(12,13,4),(12,54,0),(12,54,1),(13,54,1)]
SUBJECTS = {
    'nested-loops4': {
        'package': 'cf11', 'class': 'UpstreamNestedLoops4',
        'methods': [('testFor','()I','public int testFor();')],
        'model': 'fixed-testFor-12',
    },
    'side-exit-controls': {
        'package': 'cf11', 'class': 'NestedLoopSideExitControls',
        'methods': [('testFor','(III)I','public int testFor(int, int, int);'),
                    ('runCounts','(III)I','public int runCounts(int, int, int);')],
        'model': 'ten-case-side-exit-controls',
    },
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict:
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': sha(data)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def load_guard(path: Path, expected_sha: str):
    if sha(path.read_bytes()) != expected_sha:
        raise RuntimeError(f'guard pin mismatch: {path}')
    spec = importlib.util.spec_from_file_location('cf11_postfix_guard', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.ROOT = ROOT
    return module


def load_scanner(path: Path, expected_sha: str):
    if sha(path.read_bytes()) != expected_sha:
        raise RuntimeError(f'target-size scanner pin mismatch: {path}')
    spec = importlib.util.spec_from_file_location('cf11_postfix_scanner', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Recorder:
    def __init__(self, out: Path, guard, execution: dict):
        self.out, self.guard, self.execution = out, guard, execution
        self.commands = execution['commands']

    def run(self, label: str, argv: list[object], home: Path | None = None) -> tuple[dict, bytes, bytes]:
        command_index = len(self.commands) + 1
        string_argv = [str(value) for value in argv]
        row = {'index': command_index, 'label': label, 'argv': string_argv,
               'cwd': str(ROOT), 'java_home': str(home) if home else None,
               'state': 'started', 'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat()}
        self.commands.append(row)
        self.execution['status'] = 'running'
        write_json(self.out / 'execution.json', self.execution)
        env = {key: value for key, value in os.environ.items() if key not in STRIP_ENV}
        if home is not None:
            env['JAVA_HOME'] = str(home)
            env['PATH'] = str(home / 'bin') + os.pathsep + env.get('PATH', '')
        try:
            observed = self.guard.run_command(command_index, string_argv, env)
            row.update(observed)
            row['label'] = label
            row['java_home'] = str(home) if home else None
            row['state'] = 'observed'
            stdout_path = Path(row['streams']['stdout']['path'])
            stderr_path = Path(row['streams']['stderr']['path'])
            stdout = stdout_path.read_bytes()
            stderr = stderr_path.read_bytes()
        except BaseException as exc:
            row.update({'state': 'collector-error', 'error_type': type(exc).__name__, 'error': str(exc)})
            stdout = stderr = b''
            self.execution['status'] = 'command-error'
            self.execution['last_error'] = {'label': label, 'type': type(exc).__name__, 'message': str(exc)}
            write_json(self.out / 'execution.json', self.execution)
            raise
        write_json(self.out / 'execution.json', self.execution)
        return row, stdout, stderr


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--subject', choices=sorted(SUBJECTS), required=True)
    parser.add_argument('--source', type=Path, required=True, help='exact adapted original Java class source')
    parser.add_argument('--source-sha256', required=True, help='root-frozen SHA-256 of the exact adapted source')
    parser.add_argument('--runner', type=Path, required=True, help='exact subject Runner.java')
    parser.add_argument('--runner-sha256', required=True, help='root-frozen SHA-256 of the exact Runner')
    parser.add_argument('--upstream-source', type=Path, help='fixed upstream source; required for nested-loops4')
    parser.add_argument('--upstream-sha256', help='exact upstream source hash; required for nested-loops4')
    parser.add_argument('--cli', type=Path, required=True, help='new frozen CLI executable')
    parser.add_argument('--cli-sha256', required=True, help='root-frozen SHA-256 of the new CLI')
    parser.add_argument('--build-receipt', type=Path, required=True, help='actual receipt for the new CLI build')
    parser.add_argument('--build-receipt-sha256', required=True, help='root-frozen SHA-256 of the build receipt')
    parser.add_argument('--receipt-cli-object', required=True,
                        help='JSON key containing the CLI binding object, e.g. final_cli; root must confirm from receipt')
    parser.add_argument('--receipt-status-key', default='status', help='root-confirmed receipt status key')
    parser.add_argument('--receipt-accepted-value', required=True, help='root-confirmed accepted status value')
    parser.add_argument('--receipt-mode-key', default='mode', help='root-confirmed CLI mode field in binding object')
    parser.add_argument('--jdk-manifest', type=Path, required=True)
    parser.add_argument('--jdk-manifest-sha256', required=True)
    parser.add_argument('--jadx', type=Path, required=True)
    parser.add_argument('--jadx-sha256', required=True)
    parser.add_argument('--jadx-jar', type=Path, required=True)
    parser.add_argument('--jadx-jar-sha256', required=True)
    parser.add_argument('--guard', type=Path, required=True)
    parser.add_argument('--guard-sha256', required=True)
    parser.add_argument('--scanner', type=Path, required=True)
    parser.add_argument('--scanner-sha256', required=True)
    parser.add_argument('--classfile-parser', type=Path, required=True)
    parser.add_argument('--classfile-parser-sha256', required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()

    for path_key in ('source','runner','upstream_source','cli','build_receipt','jdk_manifest','jadx','jadx_jar',
                     'guard','scanner','classfile_parser','out'):
        value=getattr(args,path_key)
        if value is not None:
            setattr(args,path_key,value.resolve())
    out = args.out.resolve()
    if out.exists():
        raise SystemExit(f'refuse overwrite: {out}')
    if not args.cli.is_file() or not os.access(args.cli, os.X_OK):
        raise SystemExit('new frozen CLI is missing or not executable; no command run')
    if args.cli.stat().st_mode & 0o777 != 0o555:
        raise SystemExit('new frozen CLI mode is not 0555; no command run')
    if pin(args.source)['sha256'] != args.source_sha256 or pin(args.runner)['sha256'] != args.runner_sha256:
        raise SystemExit('source or Runner pin mismatch; no command run')
    if pin(args.cli)['sha256'] != args.cli_sha256:
        raise SystemExit('new CLI pin mismatch; no command run')
    if pin(args.build_receipt)['sha256'] != args.build_receipt_sha256:
        raise SystemExit('build-receipt pin mismatch; no command run')
    receipt = json.loads(args.build_receipt.read_text(encoding='utf-8'))
    binding = receipt
    for part in args.receipt_cli_object.split('.'):
        if not isinstance(binding, dict) or part not in binding:
            raise SystemExit(f'build receipt lacks configured CLI binding {args.receipt_cli_object}; no command run')
        binding = binding[part]
    if (not isinstance(binding, dict) or receipt.get(args.receipt_status_key) != args.receipt_accepted_value
            or binding.get('path') != str(args.cli.resolve())
            or binding.get('sha256') != args.cli_sha256
            or binding.get(args.receipt_mode_key) != '0o555'):
        raise SystemExit('build receipt does not bind the configured frozen CLI; no command run')
    if args.subject == 'nested-loops4' and (args.upstream_source is None or args.upstream_sha256 is None):
        raise SystemExit('nested-loops4 requires explicit upstream source and SHA; no command run')

    subject = SUBJECTS[args.subject]
    guard = load_guard(args.guard, args.guard_sha256)
    scanner = load_scanner(args.scanner, args.scanner_sha256)
    # The manifest supplies the actual JDK tool paths; every executable pin is recorded and checked.
    manifest = json.loads(args.jdk_manifest.read_text(encoding='utf-8'))
    if pin(args.jdk_manifest)['sha256'] != args.jdk_manifest_sha256:
        raise SystemExit('JDK manifest pin mismatch; no command run')
    legs = {}
    for entry in manifest.get('legs', []):
        tools = entry.get('jdk_tools', {})
        if {'java', 'javac', 'javap'} <= tools.keys():
            leg = {'name': entry['leg'], 'tools': {}, 'pins': {}, 'home': None}
            for tool, facts in tools.items():
                path = Path(facts['path'])
                actual = pin(path) if path.is_file() else None
                leg['tools'][tool] = path
                leg['pins'][tool] = {'expected_sha256': facts['sha256'], 'actual': actual,
                                     'ok': actual is not None and actual['sha256'] == facts['sha256']}
            if all(item['ok'] for item in leg['pins'].values()):
                leg['home'] = leg['tools']['java'].parent.parent
                legs[entry['leg']] = leg
    if set(legs) != {'javac8', 'javac23'}:
        raise SystemExit('expected exact pinned javac8/javac23 legs; no command run')
    for path, expected in ((args.jadx,args.jadx_sha256),(args.jadx_jar,args.jadx_jar_sha256),
                           (args.scanner,args.scanner_sha256),(args.classfile_parser,args.classfile_parser_sha256)):
        if pin(path)['sha256'] != expected:
            raise SystemExit(f'tool/parser pin mismatch: {path}; no command run')

    out.mkdir(parents=True)
    for child in ('raw','legs','inputs','jadx','jarde'):
        (out / child).mkdir()
    guard.OUT = out / 'raw'
    guard.expected_test_summaries = lambda index, stdout: None
    guard.ENV_VALUES = {}
    guard.target_bytes = lambda: scanner.target_bytes(ROOT)
    guard.command_stream = lambda path: {'path':str(Path(path).resolve()),
        'bytes':Path(path).stat().st_size,'sha256':sha(Path(path).read_bytes())}
    source = out / 'inputs' / args.source.name
    runner = out / 'inputs' / args.runner.name
    shutil.copyfile(args.source, source)
    shutil.copyfile(args.runner, runner)
    if args.subject == 'nested-loops4':
        if pin(args.upstream_source)['sha256'] != args.upstream_sha256:
            raise SystemExit('upstream source pin mismatch; no command run')
        shutil.copyfile(args.upstream_source, out / 'inputs' / args.upstream_source.name)

    execution = {
        'schema': 'cf11-nested-loop-side-exits-postfix-template-v1', 'status': 'preflight-complete',
        'subject': args.subject, 'class': subject['package'] + '.' + subject['class'],
        'target_methods': [{'name': n, 'descriptor': d} for n,d,_ in subject['methods']],
        'model': subject['model'], 'commands': [], 'expected_success_command_count': 29,
        'build_receipt': pin(args.build_receipt), 'build_receipt_cli_binding': binding,
        'cli': pin(args.cli), 'cli_mode': oct(args.cli.stat().st_mode & 0o777),
        'source_tool_pins_before': [], 'source_tool_pins_after': None,
        'ten_case_input': CASES if args.subject == 'side-exit-controls' else None,
        'resource_guard': {'minimum_free_bytes': MIN_FREE, 'maximum_target_bytes': MAX_TARGET,
                           'poll_seconds': 1, 'peak_target_sampled': True},
        'collector_config': {key: (str(value) if isinstance(value, Path) else value)
                             for key,value in vars(args).items()} | {'out': str(out)},
    }
    tracked = [Path(__file__), args.source, args.runner, args.cli, args.build_receipt,
               args.jdk_manifest, args.jadx, args.jadx_jar, args.guard, args.scanner, args.classfile_parser]
    tracked += [source, runner]
    if args.upstream_source: tracked.append(args.upstream_source)
    tracked += [Path(tool_path) for leg in legs.values() for tool_path in leg['tools'].values()]
    before = [pin(path) for path in tracked]
    execution['source_tool_pins_before'] = before
    write_json(out / 'execution.json', execution)
    rec = Recorder(out, guard, execution)

    try:
        class_dir = subject['package']
        classpath = subject['package'] + '.' + subject['class']
        expected_classes = sorted([f'{class_dir}/{subject["class"]}.class', f'{class_dir}/Runner.class'])
        originals, oracles, javap_outputs = {}, {}, {}

        def compile_case(label, sources, leg_name, oracle=None):
            leg = legs[leg_name]
            work = out / 'legs' / label
            classes, empty = work / 'classes', work / 'empty-paths'
            classes.mkdir(parents=True, exist_ok=False); empty.mkdir()
            flags = ['-source','8','-target','8'] if leg_name == 'javac8' else ['--release','8']
            comp, _, _ = rec.run(label + '-compile', [leg['tools']['javac'], *flags, '-proc:none','-g',
                '-Xlint:-options','-classpath',empty,'-sourcepath',empty,'-d',classes,*sources], leg['home'])
            run = None
            if comp.get('exit_code') == 0 and comp.get('guard_stop') is None:
                run, _, _ = rec.run(label + '-run', [leg['tools']['java'],'-Xverify:all','-cp',classes,'cf11.Runner'], leg['home'])
            class_files = sorted(p for p in classes.rglob('*.class') if p.is_file())
            actual_paths = sorted(p.relative_to(classes).as_posix() for p in class_files)
            runtime = None
            if run is not None and oracle is not None:
                rr = next(x for x in rec.commands if x['label'] == run['label'])
                stdout = Path(rr['streams']['stdout']['path']).read_bytes()
                stderr = Path(rr['streams']['stderr']['path']).read_bytes()
                runtime = {'same_jdk_original': run['exit_code'] == oracle['exit_code']
                           and stdout == oracle['stdout'] and stderr == oracle['stderr']}
            return {'label':label,'compile_command':comp['label'],'compile_exit':comp.get('exit_code'),
                    'run_command':run['label'] if run else None,'run_exit':run.get('exit_code') if run else None,
                    'source_pins':[pin(p) for p in sources], 'class_paths':actual_paths,
                    'class_set_exact':actual_paths==expected_classes,
                    'classes':[pin(p) for p in class_files], 'runtime_comparison':runtime}

        original_rows = []
        for jdk in ('javac8','javac23'):
            row = compile_case(jdk+'-original',[source,runner],jdk)
            original_rows.append(row)
            cls = out/'legs'/ (jdk+'-original')/'classes'/class_dir/(subject['class']+'.class')
            if cls.is_file(): originals[jdk] = cls
            if row['run_command']:
                runrow = next(x for x in rec.commands if x['label']==row['run_command'])
                oracles[jdk] = {'exit_code':runrow['exit_code'],
                    'stdout':Path(runrow['streams']['stdout']['path']).read_bytes(),
                    'stderr':Path(runrow['streams']['stderr']['path']).read_bytes()}
            if cls.is_file():
                jp, stdout, stderr = rec.run(jdk+'-original-javap', [legs[jdk]['tools']['javap'],'-p','-c','-s','-v',
                    '-classpath',cls.parent.parent,classpath], legs[jdk]['home'])
                javap_path=out/'legs'/(jdk+'-original')/'javap.txt'; javap_path.write_bytes(stdout)
                javap_outputs[jdk]={'command':jp['label'],'stdout_pin':pin(javap_path),'stderr_sha256':sha(stderr)}
        execution['original_compile_rows']=original_rows
        execution['original_oracles']={k:{'exit_code':v['exit_code'],'stdout_sha256':sha(v['stdout']),
                                           'stderr_sha256':sha(v['stderr'])} for k,v in oracles.items()}
        execution['javap']=javap_outputs; write_json(out/'execution.json',execution)

        jadx_input = out/'inputs'/(subject['class']+'.jar')
        if 'javac23' in originals:
            with zipfile.ZipFile(jadx_input,'w',zipfile.ZIP_STORED) as zf:
                info=zipfile.ZipInfo(f'{class_dir}/{subject["class"]}.class',date_time=(1980,1,1,0,0,0))
                info.compress_type=zipfile.ZIP_STORED
                zf.writestr(info,originals['javac23'].read_bytes())
        jadx_version, version_out, version_err = rec.run('jadx-version',[args.jadx,'--version'],legs['javac23']['home'])
        jadx_rows={}
        for profile in ('default','none'):
            destination=out/'jadx'/profile
            argv=[args.jadx,'--no-res','--config','none','--threads-count','1']
            if profile=='none': argv += ['--rename-flags','none']
            row, _, _ = rec.run('jadx-'+profile,[*argv,'-d',destination,jadx_input],legs['javac23']['home'])
            files=sorted(destination.rglob('*.java')) if destination.exists() else []
            names=[p.name for p in files]
            packages=[]
            for file in files:
                import re
                match=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',file.read_text(encoding='utf-8'))
                packages.append(match.group(1) if match else None)
            jadx_rows[profile]={'command':row['label'],'exit_code':row.get('exit_code'),
                'sources':[pin(p) for p in files],'source_names':names,'packages':packages,
                'exact_subject_source':names==[subject['class']+'.java'] and packages==[subject['package']]}
            if jadx_rows[profile]['exact_subject_source']:
                copy=out/'inputs'/('jadx-'+profile)/(subject['class']+'.java')
                copy.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(files[0],copy)
                execution.setdefault('jadx_compile_rows',[])
                for jdk in ('javac8','javac23'):
                    execution['jadx_compile_rows'].append(compile_case(jdk+'-jadx-'+profile,[copy,runner],jdk,oracles.get(jdk)))
            write_json(out/'execution.json',execution)
        execution['jadx']={'version_command':jadx_version['label'],'version_exit':jadx_version.get('exit_code'),
            'version_stdout_sha256':sha(version_out),'version_stderr_sha256':sha(version_err),
            'jar_pin':pin(jadx_input) if jadx_input.exists() else None,'profiles':jadx_rows}

        jarde_rows=[]
        for jdk in ('javac8','javac23'):
            if jdk not in originals:
                continue
            for mode in ('default','all'):
                legdir=f'{jdk}-jarde-{mode}'
                argv=[args.cli,'class-source','--input',originals[jdk],'--class',classpath,
                      '--policy','single-class','--release','8','--format','json']
                if mode=='all': argv += ['--evidence','all']
                render, raw, err=rec.run(legdir+'-render',argv,legs[jdk]['home'])
                item={'label':legdir,'mode':mode,'render_command':render['label'],
                      'render_exit':render.get('exit_code'),'render_stdout_sha256':sha(raw),
                      'render_stderr_sha256':sha(err),'render_report_path':render['streams']['stdout']['path'],
                      'compile_run':None}
                if render.get('exit_code')==0:
                    try:
                        report=json.loads(raw)
                        doc=out/'jarde'/legdir/'report.json'; write_json(doc,report)
                        java=out/'jarde'/legdir/(subject['class']+'.java')
                        java.parent.mkdir(parents=True,exist_ok=True); java.write_text(report['text'],encoding='utf-8')
                        item.update({'report_pin':pin(doc),'source_pin':pin(java),
                                     'report_execution':report.get('execution'),
                                     'method_inventory':report.get('methods')})
                        item['compile_run']=compile_case(legdir,[java,runner],jdk,oracles.get(jdk))
                    except (ValueError,TypeError,KeyError,UnicodeError) as exc:
                        item['parse_error']={'type':type(exc).__name__,'message':str(exc)}
                jarde_rows.append(item)
                execution['jarde_profiles']=jarde_rows
                write_json(out/'execution.json',execution)
        execution['jadx_compile_rows']=execution.get('jadx_compile_rows',[])
        execution['jarde_profiles']=jarde_rows
        execution['success_command_count']=len(rec.commands)
        execution['expected_success_command_count']=29
        execution['source_tool_pins_after']=[pin(path) for path in tracked]
        execution['source_tool_inputs_unchanged']=(execution['source_tool_pins_before']==execution['source_tool_pins_after'])
        execution['status']='observed' if execution['source_tool_inputs_unchanged'] else 'input-pin-changed'
        write_json(out/'execution.json',execution)
    except BaseException as exc:
        execution['status']='collector-failed'
        execution['last_error']={'type':type(exc).__name__,'message':str(exc)}
        execution['source_tool_pins_after']=[pin(path) for path in tracked if path.is_file()]
        execution['source_tool_inputs_unchanged']=(execution['source_tool_pins_before']==execution['source_tool_pins_after'])
        write_json(out/'execution.json',execution)
        raise
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
