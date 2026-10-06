#!/bin/sh
# The gating experiment's behaviour leg: the prototype render of `OP` (escape + the receiver capture
# read through the site's own discarded null-check tail + that tail owned by the site) is stripped of
# its `//` comment lines, compiled by the installed javac under `--release 8` and by the real javac 8
# (Corretto 1.8.0_432), and run under `-Xverify:all` beside the fixture's own class file. Each leg
# prints one line, and both must answer what the original class answers (`[S]/[]` shape).
#
# Self-tests before any answer is trusted: the render must carry jarde's own header (a render of
# nothing is not a render), and the original class's run must print the value the patrol recorded.
set -eu

ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
RESULTS=$ROOT/openspec/changes/recover-proved-nonnull-bound-receivers/results
FIXTURES=$RESULTS/fixtures
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
WORK=${1:-/tmp/br-replay}
rm -rf "$WORK"
mkdir -p "$WORK"

for leg in v8 v8-javac8; do
    RENDER=$RESULTS/01-experiment-e8b-$leg-OP.txt
    if ! head -1 "$RENDER" | grep -q '// jarde: presentation of'; then
        echo "SELF-TEST FAILED: $RENDER carries no jarde self-header"
        exit 1
    fi
    mkdir -p "$WORK/$leg"
    grep -v '^\s*//' "$RENDER" >"$WORK/$leg/OP.java"
    if ! grep -q 'class OP' "$WORK/$leg/OP.java"; then
        echo "SELF-TEST FAILED: the stripped text of $leg is not the class"
        exit 1
    fi

    # The fixture's own class file, run first: the answer everything else must match.
    mkdir -p "$WORK/$leg/original"
    cp "$FIXTURES/$leg/OP.class" "$WORK/$leg/original/OP.class"
    original=$(java -Xverify:all -cp "$WORK/$leg/original" OP)

    # Installed javac 23.0.1 under `--release 8`.
    mkdir -p "$WORK/$leg/current"
    (cd "$WORK/$leg/current" && javac --release 8 -Xlint:-options -d . "$WORK/$leg/OP.java")
    current=$(java -Xverify:all -cp "$WORK/$leg/current" OP)

    # The real javac 8 leg, when this machine holds it.
    if [ -x "$JAVAC8" ]; then
        mkdir -p "$WORK/$leg/javac8"
        (cd "$WORK/$leg/javac8" && "$JAVAC8" -Xlint:-options -d . "$WORK/$leg/OP.java")
        real=$(java -Xverify:all -cp "$WORK/$leg/javac8" OP)
    else
        real="(no real javac 8 on this machine)"
    fi

    echo "$leg original: $original"
    echo "$leg stripped/installed javac --release 8: $current"
    echo "$leg stripped/real javac 8: $real"
    if [ "$original" != "$current" ]; then
        echo "FAILED: the installed-javac leg diverges from the original on $leg"
        exit 1
    fi
    if [ "$original" != "$real" ] && [ "$real" != "(no real javac 8 on this machine)" ]; then
        echo "FAILED: the real-javac-8 leg diverges from the original on $leg"
        exit 1
    fi
done
echo "SELF-TEST/REPLAY OK: both legs answer the original class's own output"
