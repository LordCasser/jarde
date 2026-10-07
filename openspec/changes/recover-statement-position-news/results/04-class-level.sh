#!/bin/sh
# The behaviour leg of the corpus delta (`recover-statement-position-news`).
#
# For every class the sweep moved, this renders it with the baseline and with the patched binary,
# strips the comment lines the way the patrols' own stripped sources are made, compiles **both**
# texts with `javac --release 8` against the fixture's own sibling class files, and — when the
# patched text compiles — runs the original class and the recovered one under `-Xverify:all` and
# diffs their output. A class whose text does not compile is printed with the reason javac states,
# and the baseline's own text is compiled the same way, so a pre-existing spelling limit cannot be
# read as this change's doing.
#
# The class list is the sweep's own moved list (pass A + pass C), in its order.
#
# A text that does not compile is a *result* here, not an error: the script runs with `set -u` and
# without `-e`, and every javac status is read from a file rather than from a failing command.
set -u

ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11450-b786-7b93-b480-e65b9cc75b09
BASE=${BASE:-/tmp/jarde-spn-baseline-target/debug/jarde-cli}
PATCHED=${PATCHED:-$ROOT/target/debug/jarde-cli}
WORK=${1:-/tmp/statement-position-news/delta}
JAVAC=${JAVAC:-/usr/bin/javac}
JAVA=${JAVA:-/usr/bin/java}
for binary in "$BASE" "$PATCHED"; do
    if [ ! -x "$binary" ]; then
        echo "MISSING BINARY: $binary"
        exit 1
    fi
done
rm -rf "$WORK"
mkdir -p "$WORK"

# One moved class: its path and the name it renders under.
CLASSES="
openspec/evidence/java-syntax-2026-10-05/ctor-throw-init-patrol/fixture/CE.class CE
openspec/evidence/java-syntax-2026-10-05/discarded-allocation-patrol/fixture/DN.class DN
openspec/evidence/java-syntax-2026-10-05/discarded-allocation-patrol/fixture/CD.class CD
openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/B5.class B5
openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/B6.class B6
tests/fixtures/recover-javac8-allocation-qualifier-null-check/d3/D3.class D3
tests/fixtures/recover-statement-position-news/v8/SB.class SB
tests/fixtures/recover-statement-position-news/v8/SP.class SP
tests/fixtures/recover-statement-position-news/v8-javac8/SB.class SB
tests/fixtures/recover-statement-position-news/v8-javac8/SP.class SP
tests/fixtures/p3-nested-parent-projection/v8/Multiseg.class Multiseg
tests/fixtures/p3-nested-parent-projection/v8-javac8/Multiseg.class Multiseg
tests/fixtures/proved-java-structure/anonymous-local-decl-site-refusals/unspellable-owner-alloc/UnspellableOwnerAlloc\$1.class UnspellableOwnerAlloc\$1
"

printf '%s\n' "$CLASSES" | while IFS=' ' read -r path name; do
    [ -n "${path:-}" ] || continue
    case=$WORK/$(printf '%s' "$path" | tr '/$' '__')
    mkdir -p "$case/base" "$case/patched"
    dir=$(dirname "$ROOT/$path")
    "$BASE" class-source --policy single-class --input "$ROOT/$path" --class "$name" \
        --format text >"$case/base.txt" 2>/dev/null || true
    "$PATCHED" class-source --policy single-class --input "$ROOT/$path" --class "$name" \
        --format text >"$case/patched.txt" 2>/dev/null || true
    for side in base patched; do
        grep -v '^[[:space:]]*//' "$case/$side.txt" >"$case/$side/$name.java"
        ( cd "$case/$side" && "$JAVAC" --release 8 -nowarn -cp "$dir" -d out "$name.java" \
            >javac.log 2>&1; echo $? >javac.status )
    done
    echo "=== $path"
    echo "    baseline text compiles: $([ "$(cat "$case/base/javac.status")" = 0 ] && echo yes || echo no)"
    echo "    patched  text compiles: $([ "$(cat "$case/patched/javac.status")" = 0 ] && echo yes || echo no)"
    if [ "$(cat "$case/base/javac.status")" != 0 ]; then
        echo "    baseline javac says: $(head -2 "$case/base/javac.log" | tr '\n' ' ')"
    fi
    if [ "$(cat "$case/patched/javac.status")" = 0 ]; then
        ( cd "$case/patched" && "$JAVA" -Xverify:all -cp out:"$dir" "$name" >run.out 2>&1; echo $? >run.status )
        ( cd "$case" && "$JAVA" -Xverify:all -cp "$dir" "$name" >orig.out 2>&1; echo $? >orig.status )
        if diff -q "$case/orig.out" "$case/patched/run.out" >/dev/null; then
            echo "    behaviour: identical to the original class ($(wc -l <"$case/orig.out" | tr -d ' ') output line(s))"
        else
            echo "    behaviour: DIFFERS from the original class"
            diff "$case/orig.out" "$case/patched/run.out" | sed -n '1,6p' | sed 's/^/        /'
        fi
    else
        echo "    patched javac says: $(head -2 "$case/patched/javac.log" | tr '\n' ' ')"
        if [ "$(cat "$case/base/javac.status")" != 0 ]; then
            echo "    (the baseline text does not compile either: this class's text never was a compilable unit)"
        fi
    fi
done
