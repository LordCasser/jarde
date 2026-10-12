#!/usr/bin/env python3
"""Pin and content checks only. Deliberately never invokes a Java compiler/runtime."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent
PINS = ROOT / 'pins.json'
CASES = {
    'combined': [
        'baseline-both-empty', 'baseline-unmatched', 'baseline-five-character-match-add',
        'baseline-non-five-match-remove', 'baseline-remove-continues-and-clears',
        'baseline-null-first-list', 'baseline-null-second-list', 'baseline-null-element',
        'renamed-string-positive', 'integer-nested-add-positive', 'explicit-cast-object-boundary',
        'raw-list-method-control', 'raw-alias-heap-pollution', 'parameter-reassignment',
        'wildcard-null-write', 'try-catch-handler', 'remove-object-and-index-overloads',
    ],
    'isolated-string': [
        'string-isolated-empty', 'string-isolated-add-cme', 'string-isolated-remove-object-cme',
        'string-isolated-null-element', 'string-isolated-null-list',
    ],
    'isolated-integer': [
        'integer-isolated-empty', 'integer-isolated-add-cme', 'integer-isolated-remove-object-cme',
        'integer-isolated-null-element', 'integer-isolated-null-list', 'integer-isolated-heap-pollution',
    ],
}

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(message)

def balanced_java(text: str, label: str) -> None:
    lexical = re.sub(r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'', ' ', text, flags=re.S)
    stack = []
    pairs = {')': '(', ']': '[', '}': '{'}
    for char in lexical:
        if char in '([{':
            stack.append(char)
        elif char in ')]}':
            require(bool(stack) and stack.pop() == pairs[char], f'{label}: unbalanced {char}')
    require(not stack, f'{label}: unclosed delimiter')

pins = json.loads(PINS.read_text(encoding='utf-8'))
for rel, row in pins['files'].items():
    path = ROOT / rel
    require(path.is_file() and path.stat().st_size == row['bytes'] and sha(path) == row['sha256'], f'file pin mismatch: {rel}')
for name, row in pins['planning_artifacts'].items():
    path = Path(row['path'])
    require(path.is_file() and path.stat().st_size == row['bytes'] and sha(path) == row['sha256'], f'planning artifact changed: {name}')
for name, row in pins['v1_sources'].items():
    path = Path(row['path'])
    require(path.is_file() and path.stat().st_size == row['bytes'] and sha(path) == row['sha256'], f'v1 source changed: {name}')

all_sources = [
    ROOT/'combined/src/cf11/UpstreamNestedLoops.java', ROOT/'combined/src/cf11/Runner.java',
    ROOT/'isolated-string/src/cf11/DistinctStringFixture.java', ROOT/'isolated-string/src/cf11/Runner.java',
    ROOT/'isolated-integer/src/cf11/DistinctIntegerFixture.java', ROOT/'isolated-integer/src/cf11/Runner.java',
]
for path in all_sources:
    balanced_java(path.read_text(encoding='utf-8'), str(path.relative_to(ROOT)))
for rel, class_name, method_sig in (
    ('combined/src/cf11/UpstreamNestedLoops.java', 'UpstreamNestedLoops', 'test(List<String> l1, List<String> l2)'),
    ('isolated-string/src/cf11/DistinctStringFixture.java', 'DistinctStringFixture', 'renamed(List<String> chosen, List<String> offered)'),
    ('isolated-integer/src/cf11/DistinctIntegerFixture.java', 'DistinctIntegerFixture', 'numbers(List<Integer> selected, List<Integer> available)'),
):
    text=(ROOT/rel).read_text(encoding='utf-8')
    require(text.count('public class ') == 1 and f'public class {class_name}' in text, f'{rel}: expected single public fixture class')
    require(method_sig in text, f'{rel}: expected positive method absent')
    require('effects++;' in text, f'{rel}: real effect counter absent')
require('class DistinctStringFixture' in (ROOT/'isolated-string/src/cf11/DistinctStringFixture.java').read_text() and 'class UpstreamNestedLoops' not in (ROOT/'isolated-string/src/cf11/DistinctStringFixture.java').read_text(), 'String positive class name is not independent')
require('class DistinctIntegerFixture' in (ROOT/'isolated-integer/src/cf11/DistinctIntegerFixture.java').read_text() and 'class UpstreamNestedLoops' not in (ROOT/'isolated-integer/src/cf11/DistinctIntegerFixture.java').read_text(), 'Integer positive class name is not independent')
integer_runner=(ROOT/'isolated-integer/src/cf11/Runner.java').read_text()
require('((List) polluted).add("not-an-integer")' in integer_runner, 'heap-pollution input missing')
for variant, path in (
    ('combined', ROOT/'combined/src/cf11/Runner.java'),
    ('isolated-string', ROOT/'isolated-string/src/cf11/Runner.java'),
    ('isolated-integer', ROOT/'isolated-integer/src/cf11/Runner.java'),
):
    actual=re.findall(r'run\("([^"]+)"', path.read_text(encoding='utf-8'))
    require(actual == CASES[variant], f'{variant} case labels/order mismatch: {actual!r}')
require(sum(map(len, CASES.values())) == 28, 'oracle total mismatch')
print(json.dumps({'status':'pins_and_content_validated_no_compile','oracle_cases':28,'combined_cases':17,'string_isolated_cases':5,'integer_isolated_cases':6,'planning_artifacts':len(pins['planning_artifacts'])},ensure_ascii=False))
