#!/usr/bin/env python3
"""Build and record bounded nested-int-array compound-assignment controls.

This is an execution script for the root agent. Do not run it while reviewing this file.
It creates the canonical Java sources once, compiles the complete two-source closure with
the frozen Corretto 8/OpenJDK 23 toolchains, records javap output, then verifies only the
freshly compiled classes by running Runner under -Xverify:all.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results'
TOOLCHAIN_MANIFEST = (ROOT / 'openspec/changes/recover-bigdecimal-number-widening/results'
                      / 'number-argument-original-v1/manifest.json')
FIXTURE = ROOT / 'tests/fixtures/nested-int-array-compound-updates'
OUT = RESULTS / 'controls-v1'
ENV_KEYS_REMOVED = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
SOURCE_NAMES = ('NestedIntUpdates.java', 'Runner.java')
EXPECTED_STDOUT = (
    'ok:plain2:35\n'
    'ok:plain3:14\n'
    'ok:scalar:14\n'
    'ok:traced:123:15\n'
    'ok:outer-null:1\n'
    'ok:outer-oob:1\n'
    'ok:null-row:12\n'
    'ok:inner-oob:12\n'
    'ok:replace-row:17:100:7\n'
    'ok:different:11\n'
).encode()

NESTED_SOURCE = r'''public final class NestedIntUpdates {
    static int trace;

    public static void plain2(int[][] a, int i, int j, int x) {
        a[i][j] += x;
    }

    public static void plain3(int[][][] a, int i, int j, int k, int x) {
        a[i][j][k] += x;
    }

    public static void scalar(int[] a, int i, int x) {
        a[i] += x;
    }

    public static void traced(int[][] a, int r, int i, int x) {
        a[row(r)][index(i)] += rhs(x);
    }

    static int row(int value) {
        trace = trace * 10 + 1;
        return value;
    }

    static int index(int value) {
        trace = trace * 10 + 2;
        return value;
    }

    static int rhs(int value) {
        trace = trace * 10 + 3;
        return value;
    }

    public static int swap(int[][] a) {
        a[0] = new int[] { 100 };
        return 7;
    }

    public static int replaceRow(int[][] a) {
        a[0][0] += swap(a);
        return 7;
    }

    public static void different(int[][] a, int x) {
        a[0][0] = a[1][0] + x;
    }
}
'''

RUNNER_SOURCE = r'''public final class Runner {
    private static void require(boolean condition, String label, String value) {
        if (!condition) {
            throw new AssertionError(label + ":" + value);
        }
        System.out.println("ok:" + label + ":" + value);
    }

    public static void main(String[] args) {
        int[][] two = { { 1, 2 }, { 3, 30 } };
        NestedIntUpdates.plain2(two, 1, 1, 5);
        require(two[1][1] == 35, "plain2", Integer.toString(two[1][1]));

        int[][][] three = { { { 1, 2 }, { 3, 4 } }, { { 5, 6 }, { 7, 8 } } };
        NestedIntUpdates.plain3(three, 1, 0, 1, 8);
        require(three[1][0][1] == 14, "plain3", Integer.toString(three[1][0][1]));

        int[] one = { 9 };
        NestedIntUpdates.scalar(one, 0, 5);
        require(one[0] == 14, "scalar", Integer.toString(one[0]));

        int[][] traced = { { 10 } };
        NestedIntUpdates.trace = 0;
        NestedIntUpdates.traced(traced, 0, 0, 5);
        require(NestedIntUpdates.trace == 123 && traced[0][0] == 15,
                "traced", NestedIntUpdates.trace + ":" + traced[0][0]);

        NestedIntUpdates.trace = 0;
        boolean outerNull = false;
        try {
            NestedIntUpdates.traced(null, 0, 0, 5);
        } catch (NullPointerException expected) {
            outerNull = true;
        }
        require(outerNull && NestedIntUpdates.trace == 1,
                "outer-null", Integer.toString(NestedIntUpdates.trace));

        NestedIntUpdates.trace = 0;
        boolean outerBounds = false;
        try {
            NestedIntUpdates.traced(new int[1][1], 2, 0, 5);
        } catch (ArrayIndexOutOfBoundsException expected) {
            outerBounds = true;
        }
        require(outerBounds && NestedIntUpdates.trace == 1,
                "outer-oob", Integer.toString(NestedIntUpdates.trace));

        NestedIntUpdates.trace = 0;
        boolean nullRow = false;
        try {
            NestedIntUpdates.traced(new int[][] { null }, 0, 0, 5);
        } catch (NullPointerException expected) {
            nullRow = true;
        }
        require(nullRow && NestedIntUpdates.trace == 12,
                "null-row", Integer.toString(NestedIntUpdates.trace));

        NestedIntUpdates.trace = 0;
        boolean innerBounds = false;
        try {
            NestedIntUpdates.traced(new int[][] { { 9 } }, 0, 2, 5);
        } catch (ArrayIndexOutOfBoundsException expected) {
            innerBounds = true;
        }
        require(innerBounds && NestedIntUpdates.trace == 12,
                "inner-oob", Integer.toString(NestedIntUpdates.trace));

        int[][] replaced = { { 10 } };
        int[] oldRow = replaced[0];
        int result = NestedIntUpdates.replaceRow(replaced);
        require(result == 7 && oldRow[0] == 17 && replaced[0][0] == 100,
                "replace-row", oldRow[0] + ":" + replaced[0][0] + ":" + result);

        int[][] different = { { 2 }, { 8 } };
        NestedIntUpdates.different(different, 3);
        require(different[0][0] == 11, "different", Integer.toString(different[0][0]));
    }
}
'''


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def file_record(path: Path) -> dict:
    return {'path': str(path.relative_to(ROOT)), 'bytes': path.stat().st_size,
            'sha256': sha_file(path)}


def clean_environment() -> tuple[dict[str, str], dict[str, bool]]:
    inherited = {key: key in os.environ for key in ENV_KEYS_REMOVED}
    env = {key: value for key, value in os.environ.items() if key not in ENV_KEYS_REMOVED}
    return env, inherited


def write_capture(path_prefix: Path, stdout: bytes, stderr: bytes) -> dict:
    stdout_path = path_prefix.with_suffix('.stdout')
    stderr_path = path_prefix.with_suffix('.stderr')
    stdout_path.write_bytes(stdout)
    stderr_path.write_bytes(stderr)
    return {
        'stdout': {'path': str(stdout_path.relative_to(OUT)), 'bytes': len(stdout), 'sha256': sha_bytes(stdout)},
        'stderr': {'path': str(stderr_path.relative_to(OUT)), 'bytes': len(stderr), 'sha256': sha_bytes(stderr)},
    }


def run_command(label: str, argv: list[str], cwd: Path, env: dict[str, str], leg_root: Path) -> dict:
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            check=False)
    streams = write_capture(leg_root / label, result.stdout, result.stderr)
    return {'label': label, 'argv': argv, 'cwd': str(cwd), 'exit': result.returncode,
            'environment_removed': list(ENV_KEYS_REMOVED), **streams}


def load_frozen_toolchains() -> tuple[dict, str]:
    data = TOOLCHAIN_MANIFEST.read_bytes()
    manifest = json.loads(data)
    if manifest.get('schema') != 'bigdecimal-number-argument-original-v1':
        raise SystemExit(f'unexpected frozen JDK manifest schema: {TOOLCHAIN_MANIFEST}')
    legs = {row['leg']: row for row in manifest.get('legs', [])}
    if set(legs) != {'javac8', 'javac23'}:
        raise SystemExit('frozen JDK manifest must contain exactly javac8 and javac23')
    for leg, row in legs.items():
        for tool in ('javac', 'java', 'javap'):
            identity = row['tools'][tool]
            path = Path(identity['path'])
            if not path.is_file() or path.stat().st_size != identity['bytes'] or sha_file(path) != identity['sha256']:
                raise SystemExit(f'{leg}: frozen {tool} identity changed: {path}')
    return legs, sha_bytes(data)


def main() -> None:
    if FIXTURE.exists():
        raise SystemExit(f'refusing to overwrite canonical fixture directory: {FIXTURE}')
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite evidence directory: {OUT}')

    toolchains, toolchain_manifest_sha = load_frozen_toolchains()
    OUT.mkdir(parents=True)
    FIXTURE.mkdir(parents=True)
    fixture_sources = {
        'NestedIntUpdates.java': NESTED_SOURCE.encode(),
        'Runner.java': RUNNER_SOURCE.encode(),
    }
    for name, content in fixture_sources.items():
        (FIXTURE / name).write_bytes(content)

    env, inherited_options = clean_environment()
    manifest = {
        'schema': 'nested-int-array-compound-controls-v1',
        'status': 'started',
        'runner': file_record(Path(__file__).resolve()),
        'fixture_root': str(FIXTURE),
        'output_root': str(OUT),
        'toolchain_manifest': {'path': str(TOOLCHAIN_MANIFEST), 'sha256': toolchain_manifest_sha},
        'source_files': [file_record(FIXTURE / name) for name in SOURCE_NAMES],
        'sources_compile_together': list(SOURCE_NAMES),
        'environment_policy': {'removed_for_every_process': list(ENV_KEYS_REMOVED),
                               'inherited_at_script_start': inherited_options},
        'expected_runner_stdout': {'utf8': EXPECTED_STDOUT.decode(), 'sha256': sha_bytes(EXPECTED_STDOUT)},
        'cases': [
            {'name': 'plain2', 'expected': 'ok:plain2:35'},
            {'name': 'plain3', 'expected': 'ok:plain3:14'},
            {'name': 'scalar', 'expected': 'ok:scalar:14'},
            {'name': 'traced', 'expected': 'ok:traced:123:15', 'evaluation_order': ['row', 'index', 'rhs']},
            {'name': 'outer-null', 'expected': 'ok:outer-null:1', 'rhs_not_evaluated': True},
            {'name': 'outer-oob', 'expected': 'ok:outer-oob:1', 'rhs_not_evaluated': True},
            {'name': 'null-row', 'expected': 'ok:null-row:12', 'rhs_not_evaluated': True},
            {'name': 'inner-oob', 'expected': 'ok:inner-oob:12', 'rhs_not_evaluated': True},
            {'name': 'replace-row', 'expected': 'ok:replace-row:17:100:7',
             'old_row_value': 17, 'new_row_value': 100, 'result': 7},
            {'name': 'different', 'expected': 'ok:different:11'},
        ],
        'legs': [],
    }

    required_javap = (
        'plain2(int[][], int, int, int)', 'descriptor: ([[IIII)V',
        'plain3(int[][][], int, int, int, int)', 'descriptor: ([[[IIIII)V',
        'scalar(int[], int, int)', 'descriptor: ([III)V',
        'traced(int[][], int, int, int)', 'replaceRow(int[][])', 'different(int[][], int)',
    )
    complete = True
    for leg in ('javac8', 'javac23'):
        row = toolchains[leg]
        leg_root = OUT / leg
        classes = leg_root / 'classes'
        empty = leg_root / 'empty'
        classes.mkdir(parents=True)
        empty.mkdir()
        sources = [str(FIXTURE / name) for name in SOURCE_NAMES]
        javac = row['tools']['javac']['path']
        javap = row['tools']['javap']['path']
        java = row['tools']['java']['path']
        compile_argv = [javac, '-source', '8', '-target', '8', '-g:none', '-Xlint:-options',
                        '-classpath', str(empty), '-sourcepath', str(empty), '-d', str(classes), *sources]
        compile_record = run_command('compile', compile_argv, ROOT, env, leg_root)
        leg_record = {'leg': leg, 'jdk_tools': row['tools'], 'classes_dir': str(classes),
                      'empty_classpath_sourcepath': str(empty), 'commands': [compile_record]}
        complete = complete and compile_record['exit'] == 0
        if compile_record['exit'] == 0:
            class_path = classes / 'NestedIntUpdates.class'
            javap_argv = [javap, '-p', '-c', '-s', '-v', str(class_path)]
            javap_record = run_command('javap-nested', javap_argv, ROOT, env, leg_root)
            leg_record['commands'].append(javap_record)
            javap_bytes = (OUT / javap_record['stdout']['path']).read_bytes()
            for signature in required_javap:
                if signature.encode() not in javap_bytes:
                    leg_record.setdefault('javap_missing_signatures', []).append(signature)
            complete = complete and javap_record['exit'] == 0 and not leg_record.get('javap_missing_signatures')

            runner_argv = [java, '-Xverify:all', '-cp', str(classes), 'Runner']
            runtime_record = run_command('run-runner', runner_argv, ROOT, env, leg_root)
            leg_record['commands'].append(runtime_record)
            runtime_stdout = (OUT / runtime_record['stdout']['path']).read_bytes()
            runtime_stderr = (OUT / runtime_record['stderr']['path']).read_bytes()
            leg_record['runtime_comparison'] = {
                'exit_is_zero': runtime_record['exit'] == 0,
                'stdout_matches_oracle': runtime_stdout == EXPECTED_STDOUT,
                'stderr_is_empty': runtime_stderr == b'',
            }
            complete = complete and runtime_record['exit'] == 0 and runtime_stdout == EXPECTED_STDOUT and runtime_stderr == b''
        else:
            leg_record['commands_skipped_after_compile_failure'] = ['javap-nested', 'run-runner']
        leg_record['class_files'] = [file_record(path) for path in sorted(classes.rglob('*.class'))]
        manifest['legs'].append(leg_record)

    fixture_records = [file_record(FIXTURE / name) for name in SOURCE_NAMES]
    output_files = [file_record(path) for path in sorted(OUT.rglob('*'))
                    if path.is_file() and path.name != 'manifest.json']
    manifest['fixture_files'] = fixture_records
    manifest['output_files'] = output_files
    manifest['closed_inventory'] = fixture_records + output_files
    manifest['closed_inventory_scope'] = 'all generated fixture sources and all per-JDK evidence/class files; manifest is the inventory root and is excluded from its own listing'
    manifest['status'] = 'complete' if complete else 'recorded_failure'
    manifest['completed_unix_seconds'] = int(time.time())
    (OUT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    if not complete:
        raise SystemExit(f'control build/replay did not meet its expected result; see {OUT / "manifest.json"}')
    print(json.dumps({'status': manifest['status'], 'legs': [row['leg'] for row in manifest['legs']],
                      'fixture_files': fixture_records, 'output_root': str(OUT)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
