#!/usr/bin/env python3
"""Build and inspect the isolated handler-control fixture on frozen JDKs."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess

REPO = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = pathlib.Path(__file__).resolve().parent
OUT = RESULTS
BASELINE = REPO / 'openspec/changes/recover-constructor-primitive-conversion-arguments/results/baseline-v1'
BASELINE_MANIFEST = BASELINE / 'manifest.json'
SOURCE = REPO / 'tests/fixtures/p3-constructor-primitive-conversion-controls-v1/ConstructorPrimitiveHandlerControls.java'
CLASS_NAME = 'ConstructorPrimitiveHandlerControls'


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: pathlib.Path) -> str:
    return sha(path.read_bytes())


def main() -> int:
    if (OUT / 'manifest.json').exists():
        raise SystemExit(f'refusing to overwrite {OUT / "manifest.json"}')
    baseline = json.loads(BASELINE_MANIFEST.read_text(encoding='utf-8'))
    source_hash = file_sha(SOURCE)
    meta = {
        'schema': 'constructor-primitive-conversion-handler-controls-build-v2',
        'runner_path': str(pathlib.Path(__file__).resolve()),
        'runner_sha256': file_sha(pathlib.Path(__file__).resolve()),
        'baseline_manifest_path': str(BASELINE_MANIFEST),
        'baseline_manifest_sha256': file_sha(BASELINE_MANIFEST),
        'source_path': str(SOURCE),
        'source_bytes': SOURCE.stat().st_size,
        'source_sha256': source_hash,
        'source': SOURCE.read_text(encoding='utf-8'),
        'class_name': CLASS_NAME,
        'class_descriptor': 'LConstructorPrimitiveHandlerControls;',
        'target_method': 'sameHandler',
        'target_descriptor': '(I)Ljava/lang/Long;',
        'legs': {},
    }
    for leg, frozen in baseline['jdk_legs'].items():
        home = pathlib.Path(frozen['jdk_home'])
        javac = home / 'bin/javac'
        java = home / 'bin/java'
        javap = home / 'bin/javap'
        leg_root = OUT / leg
        classes = leg_root / 'classes'
        empty = leg_root / 'empty-classpath-sourcepath'
        classes.mkdir(parents=True, exist_ok=False)
        empty.mkdir(parents=True, exist_ok=False)
        commands = []

        def run(label: str, argv: list[str], cwd: pathlib.Path) -> dict:
            result = subprocess.run(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            stem = f'{leg}-{label}'
            stdout_path = OUT / f'{stem}.stdout'
            stderr_path = OUT / f'{stem}.stderr'
            stdout_path.write_bytes(result.stdout)
            stderr_path.write_bytes(result.stderr)
            record = {
                'label': label,
                'argv': argv,
                'cwd': str(cwd),
                'exit': result.returncode,
                'stdout': str(stdout_path.relative_to(OUT)),
                'stdout_bytes': len(result.stdout),
                'stdout_sha256': file_sha(stdout_path),
                'stderr': str(stderr_path.relative_to(OUT)),
                'stderr_bytes': len(result.stderr),
                'stderr_sha256': file_sha(stderr_path),
            }
            commands.append(record)
            return record

        compiler_flags = list(frozen['compiler_flags_from_fixture_v1_run003'])
        compile_record = run(
            'compile',
            [str(javac), *compiler_flags, '-g:none', '-classpath', str(empty),
             '-sourcepath', str(empty), '-d', str(classes), str(SOURCE)],
            leg_root,
        )
        class_file = classes / f'{CLASS_NAME}.class'
        class_record = ({'path': str(class_file.relative_to(OUT)), 'bytes': class_file.stat().st_size,
                         'sha256': file_sha(class_file)} if class_file.is_file() else None)
        javap_record = run(
            'javap',
            [str(javap), '-v', '-p', '-c', '-classpath', str(classes), CLASS_NAME],
            leg_root,
        )
        javap_text = (OUT / javap_record['stdout']).read_text(encoding='utf-8', errors='replace')
        method_start = re.search(r'^\s*static java\.lang\.Long sameHandler\(int\);\s*$', javap_text, re.M)
        method_text = javap_text[method_start.end():] if method_start else ''
        next_method = re.search(r'^\s*(?:public|private|protected|static|final|synchronized|native|abstract)\s+[^\n]+;\s*$', method_text, re.M)
        if next_method:
            method_text = method_text[:next_method.start()]
        code_line = re.search(r'^\s*stack=(\d+), locals=(\d+), args_size=(\d+)\s*$', method_text, re.M)
        exception_block = re.search(
            r'^\s*Exception table:\s*\n\s*from\s+to\s+target\s+type\s*\n((?:\s*\d+\s+\d+\s+\d+\s+[^\n]*\n?)+)',
            method_text, re.M,
        )
        handler_rows = []
        if exception_block:
            for line in exception_block.group(1).splitlines():
                fields = line.split()
                if len(fields) >= 4 and all(value.isdigit() for value in fields[:3]):
                    handler_rows.append({'start_pc': int(fields[0]), 'end_pc': int(fields[1]),
                                         'handler_pc': int(fields[2]), 'catch_type': ' '.join(fields[3:])})
        code = re.findall(r'^\s*(\d+):\s+([^\n]+)$', method_text, re.M)
        meta['legs'][leg] = {
            'jdk_home': str(home),
            'compiler_flags_from_baseline_manifest': compiler_flags,
            'tools': {name: {'path': str(path), 'sha256': file_sha(path)}
                      for name, path in [('javac', javac), ('java', java), ('javap', javap)]},
            'empty_classpath_sourcepath': str(empty),
            'compile': compile_record,
            'class_file': class_record,
            'javap': javap_record,
            'sameHandler': {
                'descriptor': 'sameHandler(I)Ljava/lang/Long;',
                'code_header': ({'max_stack': int(code_line.group(1)), 'max_locals': int(code_line.group(2)),
                                 'args_size': int(code_line.group(3))} if code_line else None),
                'exception_table': handler_rows,
                'instructions': [{'bci': int(bci), 'instruction': instruction.strip()} for bci, instruction in code],
                'full_javap_section_sha256': sha(method_text.encode()),
            },
            'commands': commands,
        }
    meta['files'] = [
        {'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size, 'sha256': file_sha(path)}
        for path in sorted(OUT.rglob('*')) if path.is_file() and path != OUT / 'manifest.json'
    ]
    (OUT / 'manifest.json').write_text(json.dumps(meta, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(json.dumps({leg: {'compile_exit': data['compile']['exit'], 'class_file': data['class_file'],
                            'descriptor': data['sameHandler']['descriptor'],
                            'code_header': data['sameHandler']['code_header'],
                            'exception_table': data['sameHandler']['exception_table']}
                      for leg, data in meta['legs'].items()}, indent=2))
    return 0 if all(leg['compile']['exit'] == 0 and leg['javap']['exit'] == 0
                    and leg['class_file'] is not None for leg in meta['legs'].values()) else 1


if __name__ == '__main__':
    raise SystemExit(main())
