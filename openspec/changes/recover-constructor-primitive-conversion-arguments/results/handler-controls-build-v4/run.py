#!/usr/bin/env python3
"""Compile and inspect the v3 handler control with the frozen two JDKs."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess

RESULTS = pathlib.Path(__file__).resolve().parent
BASELINE_MANIFEST = RESULTS.parent / 'baseline-v1' / 'manifest.json'
SOURCE = RESULTS / 'ConstructorPrimitiveHandlerControls.java'
CLASS_NAME = 'ConstructorPrimitiveHandlerControls'


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: pathlib.Path) -> str:
    return sha(path.read_bytes())


def run_command(label: str, argv: list[str], cwd: pathlib.Path, leg_root: pathlib.Path) -> dict:
    result = subprocess.run(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout_path = RESULTS / f'{leg_root.name}-{label}.stdout'
    stderr_path = RESULTS / f'{leg_root.name}-{label}.stderr'
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    return {
        'label': label,
        'argv': argv,
        'cwd': str(cwd),
        'exit': result.returncode,
        'stdout': str(stdout_path.relative_to(RESULTS)),
        'stdout_bytes': len(result.stdout),
        'stdout_sha256': file_sha(stdout_path),
        'stderr': str(stderr_path.relative_to(RESULTS)),
        'stderr_bytes': len(result.stderr),
        'stderr_sha256': file_sha(stderr_path),
    }


def inspect_javap(text: str) -> dict:
    method_start = re.search(r'^\s*static java\.lang\.Long sameHandler\(int\);\s*$', text, re.M)
    method_text = text[method_start.end():] if method_start else ''
    next_method = re.search(
        r'^\s*(?:public|private|protected|static|final|synchronized|native|abstract)\s+[^\n]+;\s*$',
        method_text, re.M)
    if next_method:
        method_text = method_text[:next_method.start()]
    code_line = re.search(r'^\s*stack=(\d+), locals=(\d+), args_size=(\d+)\s*$', method_text, re.M)
    exception_block = re.search(
        r'^\s*Exception table:\s*\n\s*from\s+to\s+target\s+type\s*\n'
        r'((?:\s*\d+\s+\d+\s+\d+\s+[^\n]*\n?)+)', method_text, re.M)
    handlers = []
    if exception_block:
        for line in exception_block.group(1).splitlines():
            fields = line.split()
            if len(fields) >= 4 and all(value.isdigit() for value in fields[:3]):
                handlers.append({
                    'start_pc': int(fields[0]), 'end_pc': int(fields[1]),
                    'handler_pc': int(fields[2]), 'catch_type': ' '.join(fields[3:]),
                })
    instructions = re.findall(r'^\s*(\d+):\s+([^\n]+)$', method_text, re.M)
    return {
        'descriptor': 'sameHandler(I)Ljava/lang/Long;',
        'code_header': ({
            'max_stack': int(code_line.group(1)), 'max_locals': int(code_line.group(2)),
            'args_size': int(code_line.group(3)),
        } if code_line else None),
        'exception_table': handlers,
        'instructions': [
            {'bci': int(bci), 'instruction': instruction.strip()}
            for bci, instruction in instructions
        ],
        'javap_method_text_sha256': sha(method_text.encode('utf-8')),
    }


def main() -> int:
    manifest_path = RESULTS / 'manifest.json'
    if manifest_path.exists():
        raise SystemExit(f'refusing to overwrite {manifest_path}')
    baseline = json.loads(BASELINE_MANIFEST.read_text(encoding='utf-8'))
    source_bytes = SOURCE.read_bytes()
    metadata = {
        'schema': 'constructor-primitive-conversion-handler-controls-build-v4',
        'runner_path': str(pathlib.Path(__file__).resolve()),
        'runner_sha256': file_sha(pathlib.Path(__file__).resolve()),
        'baseline_manifest_path': str(BASELINE_MANIFEST),
        'baseline_manifest_sha256': file_sha(BASELINE_MANIFEST),
        'source_path': str(SOURCE),
        'source_bytes': len(source_bytes),
        'source_sha256': sha(source_bytes),
        'source': source_bytes.decode('utf-8'),
        'class_name': CLASS_NAME,
        'class_descriptor': 'LConstructorPrimitiveHandlerControls;',
        'target_method': 'sameHandler',
        'target_descriptor': 'sameHandler(I)Ljava/lang/Long;',
        'legs': {},
    }

    for leg, frozen in baseline['jdk_legs'].items():
        home = pathlib.Path(frozen['jdk_home'])
        javac = home / 'bin/javac'
        javap = home / 'bin/javap'
        leg_root = RESULTS / leg
        classes = leg_root / 'classes'
        empty = leg_root / 'empty-classpath-sourcepath'
        leg_root.mkdir(exist_ok=False)
        classes.mkdir()
        empty.mkdir()
        compiler_flags = list(frozen['compiler_flags_from_fixture_v1_run003'])
        compile_record = run_command(
            'compile',
            [str(javac), *compiler_flags, '-g:none', '-classpath', str(empty),
             '-sourcepath', str(empty), '-d', str(classes), str(SOURCE)],
            leg_root, leg_root,
        )
        class_file = classes / f'{CLASS_NAME}.class'
        class_record = ({
            'path': str(class_file.relative_to(RESULTS)),
            'bytes': class_file.stat().st_size,
            'sha256': file_sha(class_file),
        } if class_file.is_file() else None)
        javap_record = run_command(
            'javap',
            [str(javap), '-v', '-p', '-c', '-classpath', str(classes), CLASS_NAME],
            leg_root, leg_root,
        )
        javap_text = (RESULTS / javap_record['stdout']).read_text(encoding='utf-8', errors='replace')
        metadata['legs'][leg] = {
            'jdk_home': str(home),
            'compiler_flags_from_baseline_manifest': compiler_flags,
            'tools': {
                name: {'path': str(tool), 'sha256': file_sha(tool)}
                for name, tool in (('javac', javac), ('javap', javap))
            },
            'empty_classpath_sourcepath': str(empty),
            'compile': compile_record,
            'class_file': class_record,
            'javap': javap_record,
            'sameHandler': inspect_javap(javap_text),
        }

    metadata['files'] = [
        {'path': str(path.relative_to(RESULTS)), 'bytes': path.stat().st_size,
         'sha256': file_sha(path)}
        for path in sorted(RESULTS.rglob('*'))
        if path.is_file() and path != manifest_path
    ]
    manifest_path.write_text(json.dumps(metadata, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(json.dumps({leg: {
        'compile_exit': row['compile']['exit'],
        'javap_exit': row['javap']['exit'],
        'class_file': row['class_file'],
        'sameHandler': row['sameHandler'],
    } for leg, row in metadata['legs'].items()}, indent=2, ensure_ascii=False))
    return 0 if all(row['compile']['exit'] == 0 and row['javap']['exit'] == 0
                    and row['class_file'] is not None for row in metadata['legs'].values()) else 1


if __name__ == '__main__':
    raise SystemExit(main())
