#!/usr/bin/env python3
"""Regenerate isolated positives from the frozen v1 legal Java source; no compiler calls."""
from __future__ import annotations
import hashlib
from pathlib import Path
import textwrap

HERE = Path(__file__).resolve().parent
V1_SOURCE = Path('/private/tmp/jarde-cf11-structured-list-controls-prep-luna-v1/src/cf11/UpstreamNestedLoops.java')

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def extract(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index('{', start)
    depth = 0
    for end in range(brace, len(source)):
        if source[end] == '{':
            depth += 1
        elif source[end] == '}':
            depth -= 1
            if depth == 0:
                return source[start:end + 1]
    raise ValueError(f'unclosed source method: {signature}')

def emit(class_name: str, method: str, destination: Path) -> bytes:
    lines = method.splitlines()
    method = '    ' + lines[0] + '\n' + textwrap.indent(textwrap.dedent('\n'.join(lines[1:])), '    ')
    content = ('package cf11;\n\nimport java.util.List;\n\npublic class ' + class_name
               + ' {\n    public static int effects;\n\n' + method + '\n}\n')
    destination.write_text(content, encoding='utf-8')
    return content.encode('utf-8')

source_bytes = V1_SOURCE.read_bytes()
source = source_bytes.decode('utf-8')
string_method = extract(source, 'public void renamed(List<String> chosen, List<String> offered)')
string_method = string_method.replace('offered.add(candidate);', 'offered.add(candidate);\n                        effects++;')
string_method = string_method.replace('chosen.remove((Object) option);', 'chosen.remove((Object) option);\n                        effects++;')
string_method = string_method.replace('chosen.clear();', 'chosen.clear();\n            effects++;')
integer_method = extract(source, 'public void numbers(List<Integer> selected, List<Integer> available)')
integer_method = integer_method.replace('available.add(value);', 'available.add(value);\n                        effects++;')
integer_method = integer_method.replace('selected.remove((Object) option);', 'selected.remove((Object) option);\n                        effects++;')
integer_method = integer_method.replace('selected.clear();', 'selected.clear();\n            effects++;')
outputs = {
    'DistinctStringFixture.java': emit('DistinctStringFixture', string_method, HERE / 'isolated-string/src/cf11/DistinctStringFixture.java'),
    'DistinctIntegerFixture.java': emit('DistinctIntegerFixture', integer_method, HERE / 'isolated-integer/src/cf11/DistinctIntegerFixture.java'),
}
for name, data in outputs.items():
    path = (HERE / ('isolated-string/src/cf11/' if 'String' in name else 'isolated-integer/src/cf11/') / name)
    print(f'{name}|{len(data)}|{sha(data)}|{path}')
