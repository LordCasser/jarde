#!/bin/sh
# The behavior leg of this change's corpus outgrowths (`recover-postfix-condition-positions`).
#
#   behavior.sh <class> <jar> <original-classpath-dir> <workdir>
#
# Renders the class with the worktree's CLI, strips the presentation's comment lines the way the
# patrols' own stripped sources were made, compiles the result with javac 23.0.1 `--release 8` and
# with real javac 8 (Corretto 1.8.0_432), and runs both under `-Xverify:all`, comparing standard
# output and the exit status with the fixture's own class file. The self-header is asserted before
# anything is counted, so a render that is not this tool's own presentation can never pass as one.
set -eu
class="$1"
jar="$2"
original="$3"
work="$4"
cli="${CLI:-/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a1139c-146e-7e82-b07b-d36e863939b3/target/debug/jarde-cli}"
javac8="${JAVAC8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac}"
java8="${JAVA8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java}"

mkdir -p "$work"
text=$("$cli" class-source --input "$jar" --class "$class" --policy plain-jar --format text 2>/dev/null)
case "$text" in
    *"// jarde: presentation of"*) ;;
    *) echo "SELF-TEST FAILED: no jarde presentation header for $class" >&2; exit 1 ;;
esac
printf '%s\n' "$text" > "$work/$class.presentation.java"
grep -v '^[[:space:]]*//' "$work/$class.presentation.java" > "$work/$class.java"
if grep -q 'jarde_refused_body' "$work/$class.java"; then
    echo "$class: STRIPPED TEXT STILL HOLDS A REFUSED BODY MARKER"
fi

run() {
    runner="$1"
    dir="$2"
    set +e
    out=$("$runner" -Xverify:all -cp "$dir" "$class" 2>/dev/null)
    status=$?
    set -e
    printf '%s\nstatus=%s\n' "$out" "$status"
}

want=$(run java "$original")
for leg in 23 8; do
    if [ "$leg" = 23 ]; then
        compiler=javac
        runner=java
        flags="--release 8"
    else
        if [ ! -x "$javac8" ]; then
            echo "$class: javac 8 leg SKIPPED (not installed at $javac8)"
            continue
        fi
        compiler="$javac8"
        runner="$java8"
        flags=""
    fi
    rm -rf "$work/out-$leg"
    mkdir -p "$work/out-$leg"
    # shellcheck disable=SC2086
    if ! "$compiler" $flags -nowarn -d "$work/out-$leg" "$work/$class.java" > "$work/javac-$leg.log" 2>&1; then
        echo "$class: javac($leg) EXIT 1"
        sed -n '1,6p' "$work/javac-$leg.log"
        continue
    fi
    got=$(run "$runner" "$work/out-$leg")
    if [ "$got" = "$want" ]; then
        echo "$class: leg $leg OK  $(printf '%s' "$got" | tr '\n' ' ')"
    else
        echo "$class: leg $leg MISMATCH  want=[$(printf '%s' "$want" | tr '\n' ' ')] got=[$(printf '%s' "$got" | tr '\n' ' ')]"
    fi
done
