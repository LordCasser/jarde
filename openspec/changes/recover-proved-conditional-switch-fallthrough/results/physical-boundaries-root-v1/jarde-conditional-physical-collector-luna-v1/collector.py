#!/usr/bin/env python3
"""Collect JVM-only observations for two offset-only full-class variants."""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough'
INPUT_ROOT = CHANGE / 'results/boundary-baseline-root-v1/jarde-conditional-boundary-preflight-root-v2/cases'
RUNNER = CHANGE / 'results/private-conditional-switch-boundaries-luna-v2/ConditionalSwitchBoundariesRunner.java'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA256 = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
RUNNER_SHA256 = 'de1955dc08a09068625035fac104b255ce48699aeb6270cae0f4f372ce26e75f'
CLASS_PINS = {
    'javac8': {'sha256': '27dea02e6dc3003a822ef7db2f447a6d21a508d1fb83c83622f86c64f4ff1b8b',
               'length': 1494, 'code_offset': 817},
    'javac23': {'sha256': 'e51368a95639b9bba3cd94a0f93ac38c11e465b984644ca28c4dc88ab6d82eb5',
                'length': 1488, 'code_offset': 811},
}
VARIANTS = {
    'A-multiple-exit': {37: (b'\x99\x00\x0d', b'\x99\x00\x14'),
                        47: (b'\xa7\x00\x1b', b'\xa7\x00\x14')},
    'B-nonadjacent': {37: (b'\x99\x00\x0d', b'\x99\x00\x1e'),
                      47: (b'\xa7\x00\x1b', b'\xa7\x00\x14')},
}
STRIP_ENV = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')
TIMEOUT_SECONDS = 60


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: Path) -> str:
    return sha(path.read_bytes())


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def need(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def u2(data: bytes, pos: int) -> int:
    return int.from_bytes(data[pos:pos + 2], 'big')


def u4(data: bytes, pos: int) -> int:
    return int.from_bytes(data[pos:pos + 4], 'big')


def method_code_start(data: bytes) -> tuple[int, int]:
    """Return Code[] start and code_length for partialBreak(II)String."""
    need(data[:4] == b'\xca\xfe\xba\xbe', 'class magic mismatch')
    cp_count = u2(data, 8)
    cp: list[bytes | None] = [None] * cp_count
    pos = 10
    i = 1
    while i < cp_count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            size = u2(data, pos)
            pos += 2
            cp[i] = data[pos:pos + size]
            pos += size
        elif tag in (3, 4):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag == 15:
            pos += 3
        else:
            raise RuntimeError(f'unsupported constant-pool tag {tag}')
        i += 1
    pos += 6  # access_flags, this_class, super_class
    pos += 2 + 2 * u2(data, pos)  # interfaces

    def skip_members(cursor: int, count: int) -> int:
        for _ in range(count):
            attr_count = u2(data, cursor + 6)
            cursor += 8
            for _ in range(attr_count):
                cursor += 2 + 4 + u4(data, cursor + 2)
        return cursor

    fields = u2(data, pos)
    pos = skip_members(pos + 2, fields)
    method_count = u2(data, pos)
    need(method_count == 5, f'expected complete five-method class, found {method_count} methods')
    pos += 2
    for _ in range(method_count):
        name_index, desc_index, attr_count = u2(data, pos + 2), u2(data, pos + 4), u2(data, pos + 6)
        name, desc = cp[name_index], cp[desc_index]
        pos += 8
        for _ in range(attr_count):
            attr_name = cp[u2(data, pos)]
            length = u4(data, pos + 2)
            info = pos + 6
            if name == b'partialBreak' and desc == b'(II)Ljava/lang/String;' and attr_name == b'Code':
                return info + 8, u4(data, info + 4)
            pos = info + length
    raise RuntimeError('target Code attribute not found')


def load_jdks() -> tuple[dict, dict]:
    need(file_sha(JDK_MANIFEST) == JDK_MANIFEST_SHA256, 'pinned JDK manifest SHA mismatch')
    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    legs = {}
    for leg in ('javac8', 'javac23'):
        row = next((r for r in manifest.get('legs', []) if r.get('leg') == leg), None)
        need(row is not None, f'manifest lacks {leg}')
        home = Path(row['jdk_tools']['java']['path']).resolve(strict=True).parent.parent
        tools = {}
        hashes = {}
        for tool in ('java', 'javac', 'javap'):
            record = row['jdk_tools'][tool]
            path = Path(record['path']).resolve(strict=True)
            expected = record['sha256']
            need(re.fullmatch(r'[0-9a-f]{64}', expected) is not None, f'{leg} {tool} manifest SHA malformed')
            need(file_sha(path) == expected, f'{leg} {tool} live SHA differs from manifest')
            need(path.parent == home / 'bin', f'{leg} {tool} outside manifest JAVA_HOME')
            tools[tool] = str(path)
            hashes[tool] = expected
        legs[leg] = {'java_home': str(home), 'tools': tools, 'tool_sha256': hashes}
    return legs, manifest


def make_variant(raw: bytes, leg: str, name: str) -> tuple[bytes, dict]:
    pin = CLASS_PINS[leg]
    need(len(raw) == pin['length'] and sha(raw) == pin['sha256'], f'{leg} original class pin mismatch')
    code_start, code_len = method_code_start(raw)
    need(code_start == pin['code_offset'] and code_len == 79, f'{leg} partialBreak Code layout mismatch')
    changed = bytearray(raw)
    diff = []
    recipe = VARIANTS[name]
    for bci, (old, new) in recipe.items():
        at = code_start + bci
        need(bytes(raw[at:at + 3]) == old, f'{leg}/{name} BCI {bci} opcode/operand mismatch')
        need(len(old) == len(new) == 3 and old[0] == new[0], 'recipe changes opcode/size')
        changed[at + 1:at + 3] = new[1:]
        diff.append({'bci': bci, 'file_offsets': [at + 1, at + 2],
                     'old_operand_hex': old[1:].hex(), 'new_operand_hex': new[1:].hex(),
                     'old_instruction_hex': old.hex(), 'new_instruction_hex': new.hex()})
    result = bytes(changed)
    actual = [i for i, (a, b) in enumerate(zip(raw, result)) if a != b]
    expected = sorted(offset for entry in diff for offset in entry['file_offsets'])
    need(actual == expected and len(actual) == 4, f'{leg}/{name} changed bytes exceed the four offsets')
    need(len(result) == len(raw), f'{leg}/{name} class length changed')
    return result, {'class_length': len(result), 'original_sha256': sha(raw),
                    'variant_sha256': sha(result), 'code_offset': code_start,
                    'code_length': code_len, 'byte_diff': diff,
                    'all_other_bytes_identical': True}


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path, help='new exclusive output directory')
    args = parser.parse_args()
    out = args.out.expanduser().resolve()
    need(not out.exists(), f'refusing to overwrite output: {out}')
    need(RUNNER.is_file() and file_sha(RUNNER) == RUNNER_SHA256, 'original Runner SHA mismatch')
    legs, _manifest = load_jdks()
    out.mkdir(parents=True)
    (out / 'streams').mkdir()
    execution = {
        'schema': 'conditional-switch-physical-collector-v1', 'status': 'collecting',
        'scope': 'JVM-observed physical class variants only', 'product_accepted': False,
        'canonical_ir_read': False, 'source_match': False,
        'source_match_note': 'No Java source corresponds to either offset-mutated variant; original Runner output is an informational baseline only.',
        'collector': {'path': str(Path(__file__).resolve()), 'sha256': file_sha(Path(__file__).resolve())},
        'jdk_manifest': {'path': str(JDK_MANIFEST), 'sha256': JDK_MANIFEST_SHA256},
        'runner_input': {'path': str(RUNNER), 'sha256': RUNNER_SHA256},
        'jdk': legs, 'commands': [], 'cases': {}, 'failures': [],
        'environment_removed': list(STRIP_ENV), 'locale': {'LC_ALL': 'C', 'TZ': 'UTC'},
        'command_timeout_seconds': TIMEOUT_SECONDS,
    }
    write_json(out / 'execution.json', execution)
    env = {k: v for k, v in os.environ.items() if k not in STRIP_ENV}
    env.update({'LC_ALL': 'C', 'TZ': 'UTC', 'LANG': 'C'})
    index = 0

    def run(label: str, leg: str, argv: list[str], cwd: Path) -> dict:
        nonlocal index
        index += 1
        row = {'index': index, 'label': label, 'leg': leg, 'argv': [str(x) for x in argv],
               'tool_path': str(Path(argv[0]).resolve()), 'tool_sha256': file_sha(Path(argv[0]).resolve()),
               'started_at': utc_now(), 'timeout_seconds': TIMEOUT_SECONDS}
        cmd_env = dict(env)
        home = legs[leg]['java_home']
        cmd_env['JAVA_HOME'] = home
        cmd_env['PATH'] = str(Path(home) / 'bin') + os.pathsep + cmd_env.get('PATH', '')
        started = time.monotonic()
        try:
            result = subprocess.run(argv, cwd=cwd, env=cmd_env, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, timeout=TIMEOUT_SECONDS, check=False)
            stdout, stderr, exit_code = result.stdout, result.stderr, result.returncode
            row['timed_out'] = False
        except subprocess.TimeoutExpired as exc:
            stdout = exc.stdout or b''
            stderr = exc.stderr or b''
            if isinstance(stdout, str): stdout = stdout.encode()
            if isinstance(stderr, str): stderr = stderr.encode()
            exit_code = None
            row['timed_out'] = True
        except OSError as exc:
            stdout, stderr, exit_code = b'', str(exc).encode('utf-8', errors='replace'), None
            row['launch_error'] = f'{type(exc).__name__}: {exc}'
            row['timed_out'] = False
        row.update({'ended_at': utc_now(), 'duration_seconds': round(time.monotonic() - started, 6),
                    'exit_code': exit_code, 'stdout_bytes': len(stdout), 'stderr_bytes': len(stderr),
                    'stdout_sha256': sha(stdout), 'stderr_sha256': sha(stderr),
                    'stdout_line_count': len(stdout.splitlines())})
        if label.endswith('-runtime'):
            row['expected_runner_records'] = 36
        stdout_path = out / 'streams' / f'{index:02d}.stdout.raw'
        stderr_path = out / 'streams' / f'{index:02d}.stderr.raw'
        stdout_path.write_bytes(stdout)
        stderr_path.write_bytes(stderr)
        row['stdout_path'] = str(stdout_path.relative_to(out))
        row['stderr_path'] = str(stderr_path.relative_to(out))
        execution['commands'].append(row)
        write_json(out / 'execution.json', execution)
        return row

    try:
        # A JVM-observed original baseline is captured for context; variants have no source oracle.
        for leg in ('javac8', 'javac23'):
            original = INPUT_ROOT / leg / 'original/classes/ConditionalSwitchBoundaries.class'
            raw = original.read_bytes()
            need(sha(raw) == CLASS_PINS[leg]['sha256'], f'{leg} frozen original class SHA mismatch')
            leg_case = execution['cases'].setdefault(leg, {})
            base = out / 'cases' / leg / 'original'
            classes = base / 'classes'
            classes.mkdir(parents=True)
            shutil.copyfile(original, classes / original.name)
            runner_copy = base / RUNNER.name
            shutil.copyfile(RUNNER, runner_copy)
            java, javac, javap = (legs[leg]['tools'][x] for x in ('java', 'javac', 'javap'))
            compile_row = run(f'{leg}-original-runner-compile', leg,
                [javac, '-J-Duser.language=en', '-J-Duser.country=US', '-encoding', 'UTF-8',
                 '-proc:none', '-classpath', str(classes), '-d', str(classes), str(runner_copy)], out)
            runtime_row = None
            if compile_row['exit_code'] == 0 and not compile_row.get('timed_out'):
                runtime_row = run(f'{leg}-original-runtime', leg,
                    [java, '-Xverify:all', '-cp', str(classes), 'ConditionalSwitchBoundariesRunner'], out)
                shutil.copyfile(out / runtime_row['stdout_path'], out / 'streams' / f'{leg}-original.stdout.raw')
                shutil.copyfile(out / runtime_row['stderr_path'], out / 'streams' / f'{leg}-original.stderr.raw')
            leg_case['original'] = {'class_sha256': sha(raw), 'compile': compile_row, 'runtime': runtime_row}

            for variant_name in VARIANTS:
                mutated, facts = make_variant(raw, leg, variant_name)
                case = out / 'cases' / leg / variant_name
                vclasses = case / 'classes'
                vclasses.mkdir(parents=True)
                class_path = vclasses / original.name
                class_path.write_bytes(mutated)
                runner_path = case / RUNNER.name
                shutil.copyfile(RUNNER, runner_path)
                facts.update({'class_path': str(class_path.relative_to(out)),
                              'runner_path': str(runner_path.relative_to(out)),
                              'runner_sha256': file_sha(runner_path),
                              'copied_full_class': True})
                vrow = {'variant': facts, 'javap': None, 'compile_original_runner': None,
                        'runtime': None, 'source_oracle_relation': 'no-matching-source-informational-comparison-only'}
                # Show the complete transformed class and descriptors before invoking its JVM behavior.
                vrow['javap'] = run(f'{leg}-{variant_name}-javap', leg,
                    [javap, '-c', '-p', '-s', str(class_path)], out)
                vrow['compile_original_runner'] = run(f'{leg}-{variant_name}-runner-compile', leg,
                    [javac, '-J-Duser.language=en', '-J-Duser.country=US', '-encoding', 'UTF-8',
                     '-proc:none', '-classpath', str(vclasses), '-d', str(vclasses), str(runner_path)], out)
                if vrow['compile_original_runner']['exit_code'] == 0 and not vrow['compile_original_runner'].get('timed_out'):
                    vrow['runtime'] = run(f'{leg}-{variant_name}-runtime', leg,
                        [java, '-Xverify:all', '-cp', str(vclasses), 'ConditionalSwitchBoundariesRunner'], out)
                # Retain runtime outputs verbatim; a JVM verifier rejection is evidence, not acceptance.
                if vrow['runtime'] is not None:
                    vrow['jvm_exit_code'] = vrow['runtime']['exit_code']
                    vrow['jvm_verification_observed'] = not vrow['runtime'].get('timed_out', False)
                    vrow['jvm_outcome'] = ('exit-0' if vrow['runtime']['exit_code'] == 0 else
                                           'jvm-failed-or-nonzero-exit')
                leg_case[variant_name] = vrow
                write_json(out / 'execution.json', execution)
        execution['status'] = 'jvm-observed'
        execution['finished_at'] = utc_now()
    except BaseException as exc:
        execution['status'] = 'incomplete'
        execution['failures'].append(f'{type(exc).__name__}: {exc}')
        execution['finished_at'] = utc_now()
        write_json(out / 'execution.json', execution)
        raise
    write_json(out / 'execution.json', execution)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
