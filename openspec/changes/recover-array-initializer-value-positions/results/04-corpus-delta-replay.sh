#!/bin/sh
# The corpus delta's own replay: every class the sweep moved **besides** this change's own fixtures
# is re-rendered, and the member that moved (its `main` in all twelve cases) is compiled and run
# beside the fixture's own class and compared with what that class prints.
#
# The unit is a subclass: `public class XR extends X { <the recovered main body> }`, compiled with
# the fixture's own `X.class` on the class path. An unqualified static call inside a subclass
# resolves through inheritance, so the recovered text calls the very methods the fixture's own
# `main` calls, and the two runs must print the same line. Both compilers this machine holds are
# used, and every run is under `-Xverify:all`.
set -eu
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
CLI=${CLI:-$ROOT/target/debug/jarde-cli}
WORK=${1:-/tmp/array-value/delta}
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java
rm -rf "$WORK"
mkdir -p "$WORK"

# class|render input (jar or loose class)|container entry or empty
CASES='
tests/fixtures/recover-loop-else-if-early-returns/v8/BS.class|
tests/fixtures/recover-loop-else-if-early-returns/v8/CB.class|
tests/fixtures/recover-loop-else-if-early-returns/v8/CB2.class|
tests/fixtures/recover-loop-else-if-early-returns/v8/LB.class|
tests/fixtures/recover-loop-else-if-early-returns/v8-javac8/BS.class|
tests/fixtures/recover-loop-else-if-early-returns/v8-javac8/CB.class|
tests/fixtures/recover-loop-else-if-early-returns/v8-javac8/CB2.class|
tests/fixtures/recover-loop-else-if-early-returns/v8-javac8/LB.class|
openspec/evidence/java-syntax-2026-10-05/assign-chain-soundness-patrol/fixture/ca2.jar|CA2
openspec/evidence/java-syntax-2026-10-05/postinc-condition-patrol/fixture/cp7.jar|CP7
openspec/evidence/java-syntax-2026-10-05/triple-nested-labels-patrol/fixture/nl.jar|NL
openspec/evidence/java-syntax-2026-10-05/binary-search-twopointer-patrol/fixture/bs.jar|BS
'

printf '%s\n' "$CASES" | while IFS='|' read -r input entry; do
    [ -n "$input" ] || continue
    case "$input" in
        *.jar) class=$entry; policy=plain-jar ;;
        *) class=$(basename "$input" .class); policy=single-class ;;
    esac
    dir=$WORK/$class.$(basename "$input")
    mkdir -p "$dir/original" "$dir/unit"
    case "$policy" in
        plain-jar) (cd "$dir/original" && unzip -o -q "$ROOT/$input") ;;
        *) cp "$ROOT/$input" "$dir/original/$class.class" ;;
    esac
    "$CLI" class-source --policy "$policy" --input "$ROOT/$input" --class "$class" \
        --format text >"$dir/$class.txt" 2>/dev/null || true
    head -1 "$dir/$class.txt" | grep -q '// jarde: presentation of' || {
        echo "FAILED: $class renders nothing"; exit 1; }
    python3 - "$dir/$class.txt" "$dir/unit/XR.java" "$class" <<'PY'
import sys, pathlib
text, out, class_name = sys.argv[1], sys.argv[2], sys.argv[3]
lines = pathlib.Path(text).read_text().splitlines()
start = next(i for i, line in enumerate(lines) if line.startswith('    public static void main'))
end = next(i for i in range(start, len(lines)) if lines[i] == '    }')
body = [line for line in lines[start:end + 1] if not line.strip().startswith('//')]
assert body[0].startswith('    public static void main'), body[0]
pathlib.Path(out).write_text("public class XR extends %s {\n%s\n}\n" % (class_name, "\n".join(body)))
PY
    for pair in "release:/usr/bin/javac:/usr/bin/java:--release" "javac8:$JAVAC8:$JAVA8:"; do
        label=${pair%%:*}; rest=${pair#*:}
        compiler=$(printf '%s' "$rest" | cut -d: -f1)
        runner=$(printf '%s' "$rest" | cut -d: -f2)
        release=$(printf '%s' "$rest" | cut -d: -f3)
        [ -x "$compiler" ] || continue
        if [ -n "$release" ]; then
            "$compiler" --release 8 -nowarn -cp "$dir/original" -d "$dir/unit" "$dir/unit/XR.java"
        else
            "$compiler" -nowarn -cp "$dir/original" -d "$dir/unit" "$dir/unit/XR.java"
        fi
        got=$("$runner" -Xverify:all -cp "$dir/unit:$dir/original" XR)
        want=$("$runner" -Xverify:all -cp "$dir/original" "$class")
        if [ "$got" != "$want" ]; then
            echo "FAILED: $class via $label: recovered main printed [$got], its own class [$want]"
            exit 1
        fi
        echo "OK: $class via $label -> $got"
    done
done
echo "DELTA REPLAY OK: every moved driver prints what its own class prints, on both compilers"
