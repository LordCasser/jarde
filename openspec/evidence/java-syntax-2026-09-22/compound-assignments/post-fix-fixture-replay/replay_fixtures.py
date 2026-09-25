"""Replay whole-class and verifier-safe compound-lvalue boundary evidence."""

from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPLAY = Path(__file__).resolve().parent
ROOT = REPLAY.parents[4]
FIXTURE = ROOT / 'tests' / 'fixtures' / 'p3-compound-lvalue-updates'
EVIDENCE = REPLAY / 'evidence'
PATCHER = REPLAY.parent / 'permanent-fixture-replay' / 'patch_boundaries.py'
CLI = None
EXPECTED_CLI = None
BOUNDARY_MODES = {
    'field-different-member': 'field-member',
    'array-different-index-copy': 'array-index',
    'array-different-array-copy': 'array-array',
    'field-multi-consumer': 'field-extra',
    'array-multi-consumer': 'array-extra',
    'field-gap-before-dup': 'field-snapshot',
    'field-gap-after-dup': 'field-snapshot',
    'field-gap-before-store': 'field-snapshot',
    'array-gap-before-index': 'array-snapshot',
    'array-gap-after-dup': 'array-snapshot',
    'array-gap-before-store': 'array-snapshot',
}
BASE_MODES = (
    'ordinary-field', 'ordinary-array', 'wide-field', 'wide-array',
    'field-snapshot', 'array-snapshot', 'array-null', 'array-bounds',
)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def package(source):
    match = re.search(r'^package\s+([^;]+);', source, re.MULTILINE)
    return match.group(1) if match else ''


def logged(directory, name, args):
    directory.mkdir(parents=True, exist_ok=True)
    process = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (directory / f'{name}.stdout').write_text(process.stdout)
    (directory / f'{name}.stderr').write_text(process.stderr)
    (directory / f'{name}.status').write_text(str(process.returncode) + '\n')
    return process


def compile_and_run(work, label, generated_source, helper_names, log_dir):
    source_text = generated_source.read_text()
    pkg = package(source_text)
    output = work / f'{label}-classes'
    output.mkdir(parents=True, exist_ok=True)
    primary = output / generated_source.name
    shutil.copy2(generated_source, primary)
    supports = []
    for name in helper_names:
        original = FIXTURE / name
        target = output / name
        target.write_text((f'package {pkg};\n\n' if pkg else '') + original.read_text())
        supports.append(str(target))
    compiled = logged(log_dir, f'{label}-javac',
                      ['javac', '--release', '8', '-g:none', '-d', str(output),
                       str(primary), *supports])
    result = {'javac': compiled.returncode}
    if compiled.returncode == 0:
        main = 'CompoundRunner' if 'CompoundRunner.java' in helper_names else 'CompoundBoundaryRunner'
        main_class = f'{pkg}.{main}' if pkg else main
        result['runtime'] = None
        result['class_dir'] = str(output)
        result['main_class'] = main_class
    return result


def execute(work, result, label, args, log_dir):
    if result.get('javac') != 0:
        return
    process = logged(log_dir, f'{label}-runtime',
                     ['java', '-Xverify:all', '-cp', result['class_dir'],
                      result['main_class'], *args])
    result['runtime'] = process.returncode
    result['output'] = process.stdout.splitlines()


def decompile_and_compile(work, label, class_file, helper_names, log_dir, jadx):
    case = log_dir
    case.mkdir(parents=True, exist_ok=True)
    if jadx:
        output_dir = work / f'{label}-jadx'
        decompiled = logged(case, f'{label}-jadx',
                            ['jadx', '--no-res', '-d', str(output_dir), str(class_file)])
        sources = list(output_dir.rglob(f'{class_file.stem}.java')) if decompiled.returncode == 0 else []
        generated_source = sources[0] if sources else None
        if generated_source:
            target = case / 'jadx.java.txt'
            shutil.copy2(generated_source, target)
        else:
            return {'decompile': decompiled.returncode, 'javac': 'skipped'}
    else:
        generated = subprocess.run(
            [str(CLI), 'class-source', '--input', str(class_file), '--class', class_file.stem,
             '--policy', 'single-class', '--release', '8', '--format', 'text'],
            capture_output=True, text=True, timeout=90)
        (case / 'jarde.java.txt').write_text(generated.stdout)
        (case / 'jarde-report.txt').write_text(generated.stderr)
        (case / 'jarde.status').write_text(str(generated.returncode) + '\n')
        if generated.returncode != 0:
            return {'cli': generated.returncode, 'javac': 'skipped'}
        generated_source = work / f'{label}-input' / f'{class_file.stem}.java'
        generated_source.parent.mkdir(parents=True, exist_ok=True)
        generated_source.write_text(generated.stdout)

    result = compile_and_run(work, 'jadx' if jadx else 'jarde', generated_source,
                             helper_names, case)
    result['decompile'] = 0 if jadx else None
    return result


def compare_case(work, name, mode, class_file, original_class_dir, helpers, case_dir):
    original_log = case_dir / 'original'
    javap = logged(original_log, 'javap',
                   ['javap', '-c', '-p', '-classpath', str(class_file.parent), class_file.stem])
    original = logged(original_log, 'runtime',
                      ['java', '-Xverify:all', '-cp', str(original_class_dir),
                       'CompoundBoundaryRunner', mode])
    jadx = decompile_and_compile(work, name, class_file, helpers, case_dir / 'jadx', True)
    if jadx.get('javac') == 0:
        execute(work, jadx, 'jadx', [mode], case_dir / 'jadx')
    jarde = decompile_and_compile(work, name, class_file, helpers, case_dir / 'jarde', False)
    if jarde.get('javac') == 0:
        execute(work, jarde, 'jarde', [mode], case_dir / 'jarde')
    original_lines = original.stdout.splitlines()
    for result in (jadx, jarde):
        if result.get('runtime') == 0:
            result['equal_to_original'] = result['output'] == original_lines
    return {
        'original_javap': javap.returncode,
        'original_runtime': original.returncode,
        'original_output': original_lines,
        'jadx': jadx,
        'jarde': jarde,
    }


def compare_base_modes(work, modes, class_dir, helpers):
    class_file = class_dir / 'CompoundBoundaryProbe.class'
    shared = EVIDENCE / 'boundaries' / 'base-class'
    original_log = shared / 'original'
    javap = logged(original_log, 'javap',
                   ['javap', '-c', '-p', '-classpath', str(class_dir), class_file.stem])
    jadx = decompile_and_compile(work, 'boundary-base', class_file, helpers,
                                 shared / 'jadx', True)
    if jadx.get('javac') == 0:
        jadx_base = dict(jadx)
    else:
        jadx_base = None
    jarde = decompile_and_compile(work, 'boundary-base', class_file, helpers,
                                  shared / 'jarde', False)
    jarde_base = dict(jarde) if jarde.get('javac') == 0 else None
    result = {}
    for mode in modes:
        case_dir = EVIDENCE / 'boundaries' / mode
        per_case_original = case_dir / 'original'
        original = logged(per_case_original, 'runtime',
                          ['java', '-Xverify:all', '-cp', str(class_dir),
                           'CompoundBoundaryRunner', mode])
        case = {
            'original_javap': javap.returncode,
            'original_runtime': original.returncode,
            'original_output': original.stdout.splitlines(),
        }
        for label, base in (('jadx', jadx_base), ('jarde', jarde_base)):
            if base is None:
                case[label] = dict(jadx if label == 'jadx' else jarde)
                continue
            tool_result = dict(base)
            execute(work, tool_result, label, [mode], case_dir / label)
            if tool_result.get('runtime') == 0:
                tool_result['equal_to_original'] = (
                    tool_result['output'] == original.stdout.splitlines())
            case[label] = tool_result
        result[mode] = case
    return result


def run_main_fixture(work):
    case_dir = EVIDENCE / 'whole-class'
    case_dir.mkdir(parents=True, exist_ok=True)
    original_dir = work / 'source-classes'
    original_dir.mkdir()
    sources = [FIXTURE / name for name in ('CompoundProbe.java', 'CompoundBox.java', 'CompoundRunner.java')]
    compile_source = logged(case_dir, 'original-javac',
                            ['javac', '--release', '8', '-g:none', '-d', str(original_dir),
                             *map(str, sources)])
    class_file = original_dir / 'CompoundProbe.class'
    checked_class = FIXTURE / 'v8' / 'CompoundProbe.class'
    assert sha256(class_file) == sha256(checked_class)
    javap = logged(case_dir, 'original-javap',
                   ['javap', '-c', '-p', str(class_file)])
    code_methods = javap.stdout.count('    Code:')
    original = logged(case_dir, 'original-runtime',
                      ['java', '-Xverify:all', '-cp', str(original_dir), 'CompoundRunner'])
    assert compile_source.returncode == 0 and original.returncode == 0 and code_methods == 12
    helpers = ('CompoundBox.java', 'CompoundRunner.java')
    jadx = decompile_and_compile(work, 'main', class_file, helpers, case_dir / 'jadx', True)
    if jadx.get('javac') == 0:
        execute(work, jadx, 'jadx', [], case_dir / 'jadx')
    jarde = decompile_and_compile(work, 'main', class_file, helpers, case_dir / 'jarde', False)
    if jarde.get('javac') == 0:
        execute(work, jarde, 'jarde', [], case_dir / 'jarde')
    expected = original.stdout.splitlines()
    return {
        'source_hashes': {name: sha256(FIXTURE / name)
                          for name in ('CompoundProbe.java', 'CompoundBox.java', 'CompoundRunner.java')},
        'class_bytes': class_file.stat().st_size,
        'class_sha256': sha256(class_file),
        'code_methods': code_methods,
        'original_javac': compile_source.returncode,
        'original_runtime': original.returncode,
        'original_output': expected,
        'jadx': jadx,
        'jarde': jarde,
        'jadx_equal': jadx.get('output') == expected if jadx.get('runtime') == 0 else None,
        'jarde_equal': jarde.get('output') == expected if jarde.get('runtime') == 0 else None,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, default=os.environ.get('JARDE_COMPOUND_CLI'),
                        help='CLI path (defaults to JARDE_COMPOUND_CLI)')
    parser.add_argument('--cli-sha256', default=os.environ.get('JARDE_COMPOUND_CLI_SHA256'),
                        help='optional expected CLI SHA-256')
    parser.add_argument('--evidence-dir', type=Path,
                        default=os.environ.get('JARDE_COMPOUND_EVIDENCE_DIR'),
                        help='output directory (defaults to this script\'s evidence/ directory)')
    args = parser.parse_args()
    if args.cli is None:
        parser.error('pass --cli or set JARDE_COMPOUND_CLI')
    global CLI, EXPECTED_CLI, EVIDENCE
    CLI = args.cli.resolve()
    EXPECTED_CLI = args.cli_sha256
    if args.evidence_dir is not None:
        EVIDENCE = args.evidence_dir.resolve()
    else:
        EVIDENCE = EVIDENCE.resolve()
    old_evidence = REPLAY.parent / 'permanent-fixture-replay' / 'evidence'
    assert EVIDENCE != old_evidence.resolve(), 'post-fix replay must preserve pre-change evidence'
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    cli_before = sha256(CLI)
    if EXPECTED_CLI is not None:
        assert cli_before == EXPECTED_CLI, (cli_before, EXPECTED_CLI)
    summary = {
        'cli_path': str(CLI),
        'cli_sha256_expected': EXPECTED_CLI,
        'cli_sha256_before': cli_before,
        'whole_class': None,
        'boundaries': {},
    }
    with tempfile.TemporaryDirectory(prefix='jarde-compound-fixture-') as raw:
        work = Path(raw)
        summary['whole_class'] = run_main_fixture(work)
        boundary_source_dir = work / 'boundary-source-classes'
        boundary_source_dir.mkdir()
        boundary_sources = [FIXTURE / name for name in (
            'BoundaryBox.java', 'CompoundBoundaryProbe.java', 'CompoundBoundaryRunner.java')]
        boundary_compile = logged(EVIDENCE / 'boundaries' / 'base-class' / 'original',
                                  'source-javac',
                                  ['javac', '--release', '8', '-g:none', '-d',
                                   str(boundary_source_dir), *map(str, boundary_sources)])
        boundary_class = boundary_source_dir / 'CompoundBoundaryProbe.class'
        checked_boundary_class = FIXTURE / 'boundaries' / 'v8' / 'CompoundBoundaryProbe.class'
        assert boundary_compile.returncode == 0
        assert sha256(boundary_class) == sha256(checked_boundary_class)
        boundary_javap = logged(EVIDENCE / 'boundaries' / 'base-class' / 'original',
                                'source-javap', ['javap', '-c', '-p', str(boundary_class)])
        summary['boundary_fixture'] = {
            'source_hashes': {name: sha256(FIXTURE / name) for name in (
                'BoundaryBox.java', 'CompoundBoundaryProbe.java', 'CompoundBoundaryRunner.java')},
            'class_bytes': boundary_class.stat().st_size,
            'class_sha256': sha256(boundary_class),
            'code_methods': boundary_javap.stdout.count('    Code:'),
            'javac': boundary_compile.returncode,
            'javap': boundary_javap.returncode,
        }
        subprocess.run([sys.executable, str(PATCHER)],
                       check=True, capture_output=True, text=True)
        boundary_root = FIXTURE / 'boundaries'
        base_dir = boundary_root / 'v8'
        base_class = base_dir / 'CompoundBoundaryProbe.class'
        helpers = ('BoundaryBox.java', 'CompoundBoundaryRunner.java')
        cases = [(name, mode, boundary_root / 'patched' / name / 'CompoundBoundaryProbe.class')
                 for name, mode in BOUNDARY_MODES.items()]
        for name, mode, class_file in cases:
            case_dir = EVIDENCE / 'boundaries' / name
            original_classpath = os.pathsep.join((str(class_file.parent), str(base_dir)))
            summary['boundaries'][name] = compare_case(
                work, name, mode, class_file, original_classpath, helpers, case_dir)
        summary['boundaries'].update(
            compare_base_modes(work, BASE_MODES, base_dir, helpers))
    cli_after = sha256(CLI)
    assert cli_after == cli_before
    summary['cli_sha256_after'] = cli_after
    (EVIDENCE / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
