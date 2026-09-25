"""Replay null-resource refusal and ordinary-catch classification with the final CLI."""

import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
CLI = Path('/tmp/jarde-cli-null-root-final')
EXPECTED = '30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c'
CASES = {
    'broken-suppression': ('broken-suppression/v8/NullResourceCore.class', 'NullResourceCore', 'useNullResource'),
    'inherited-only': ('inherited/v8/NullResourceInheritedOnly.class', 'NullResourceInheritedOnly', 'useNullResource'),
    'ordinary-catch': ('same-type/v8/NullResourceCore.class', 'NullResourceCore', 'ordinaryNullThenCatch'),
}


def fragment(source, method):
    marker = f'    public static void {method}('
    start = source.index(marker)
    end = source.find('\n    public ', start + len(marker))
    return source[start:end if end >= 0 else None]


def main():
    cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert cli_hash == EXPECTED
    result = {'cli_sha256': cli_hash, 'cases': {}}
    for label, (relative, class_name, method_name) in CASES.items():
        source = ROOT / 'tests/fixtures/p3-null-resource' / relative
        out = HERE / label
        out.mkdir(exist_ok=True)
        args = [str(CLI), 'class-source', '--input', str(source), '--class', class_name, '--policy', 'single-class', '--release', '8', '--format', 'text', '--evidence', 'all']
        generated = subprocess.run(args, capture_output=True, text=True, timeout=45)
        (out / 'jarde.java.txt').write_text(generated.stdout)
        (out / 'jarde-report.txt').write_text(generated.stderr)
        javap = subprocess.run(['javap', '-c', '-v', '-p', str(source)], capture_output=True, text=True, timeout=45)
        (out / 'javap.txt').write_text(javap.stdout + javap.stderr)
        method = fragment(generated.stdout, method_name)
        result['cases'][label] = {
            'class_bytes': source.stat().st_size,
            'class_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
            'cli_exit': generated.returncode,
            'javap_exit': javap.returncode,
            'method': method_name,
            'method_has_resource': 'try (' in method,
            'method_has_runtime_catch': 'catch (java.lang.RuntimeException' in method,
            'method_has_throwable_catch': 'catch (java.lang.Throwable' in method,
            'method_has_fallback': '// @bytecode' in method,
            'suppression_refusal': 'jre_guard_suppressed' in generated.stderr,
            'direct_interface_refusal': 'does not directly declare `java/lang/AutoCloseable`' in method,
        }
    assert hashlib.sha256(CLI.read_bytes()).hexdigest() == EXPECTED
    (HERE / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
