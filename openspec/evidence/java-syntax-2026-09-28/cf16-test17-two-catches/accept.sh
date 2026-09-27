#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
CLI="$1"
OUT="$2"
PACKAGE=jadx/tests/integration/trycatch
TARGET='TestTryCatchFinally17$TestCls'

"$HERE/replay.sh" "$CLI" "$OUT"
[[ "$(cat "$OUT/jarde-status.txt")" == target_status=recovered ]]
cmp "$OUT/probe-classes/$PACKAGE/$TARGET.class" "$HERE/classes/Test17.class"

python3 - "$HERE/probe/Runner.java" "$OUT/jarde-src/Runner.java" <<'PY'
from pathlib import Path
import sys
source = Path(sys.argv[1]).read_text()
assert source.count('TestTryCatchFinally17.TestCls') == 1
# Jarde presents each physical class as a top-level Java class with its binary name.
Path(sys.argv[2]).write_text(source.replace('TestTryCatchFinally17.TestCls', 'TestTryCatchFinally17$TestCls'))
PY
mkdir -p "$OUT/jarde-classes"
javac --release 8 -g -Xlint:-options -d "$OUT/jarde-classes" "$OUT/jarde-src/"*.java
java -Xverify:all -cp "$OUT/jarde-classes" jadx.tests.integration.trycatch.Runner > "$OUT/jarde.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jadx.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jarde.run.txt"

"$CLI" class-source --input "$OUT/probe-classes/$PACKAGE/$TARGET.class" \
  --class "$PACKAGE/$TARGET" --policy single-class --release 8 --format json \
  > "$OUT/jarde-target.json" 2> "$OUT/jarde-target-json.report.txt"
python3 - "$OUT/jarde-target.json" <<'PY'
import json
import sys
report = json.load(open(sys.argv[1]))
method = next(item for item in report['methods'] if item['item']['name']['escaped'] == 'test')
recovered = method['outcome']['report']
text = recovered['text']
assert method['outcome']['kind'] == 'recovered'
assert recovered['fallbacks'] == []
assert text.count('finally {') == 1
assert text.count('doFinally();') == 1
assert text.count('catch (') == 2
assert 'catch (java.lang.UnsupportedOperationException e) {\n    }' in text
assert 'catch (java.lang.NullPointerException e) {\n        return 1;' in text
origins = set()
for segment in recovered['source_map']['segments']:
    origin = segment['origin']
    origins.add(origin['primary']['bci'])
    origins.update(item['bci'] for item in origin['derived'])
assert origins == {0, 3, 6, 9, 10, 13, 16, 17, 18, 19, 22, 23, 24, 25, 28, 29, 30, 31}, origins
PY

python3 "$HERE/neighbors.py" "$OUT/probe-classes" "$OUT/neighbors"
python3 - "$HERE" "$OUT" "$CLI" <<'PY'
from pathlib import Path
import json
import subprocess
import sys
here, out, cli = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
for name in ('different-target', 'cleanup-covered', 'cleanup-self-protected',
             'rows-swapped', 'saved-return-rewritten', 'throwable-rewritten',
             'external-cleanup-entry'):
    generated = out / 'neighbors' / name / 'jadx/tests/integration/trycatch/TestTryCatchFinally17$TestCls.class'
    assert generated.read_bytes() == (here / 'classes' / 'near' / (name + '.class')).read_bytes(), name
    result = subprocess.run([cli, 'class-source', '--input', str(generated),
        '--class', 'jadx/tests/integration/trycatch/TestTryCatchFinally17$TestCls',
        '--policy', 'single-class', '--release', '8', '--format', 'json'],
        capture_output=True, text=True, check=True)
    report = json.loads(result.stdout)
    method = next(item for item in report['methods'] if item['item']['name']['escaped'] == 'test')
    text = method['outcome']['report']['text']
    assert '@bytecode' in text and 'finally {' not in text, name
print('original, pinned JADX, and Jarde eight-path Java 8 replay passed; seven verified neighbors refused')
PY
