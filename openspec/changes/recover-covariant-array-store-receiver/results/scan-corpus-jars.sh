#!/bin/sh
# Fixture **family-jar** two-leg scan (change `recover-double-brace-allocation-site`, task 3.1).
#
# `scan-corpus.sh` renders every fixture `.class` through `--policy single-class`, which is the
# right reading for a presentation that only needs the class it renders. The double-brace
# allocation point needs the *companion* too: a single-class input holds no child, so that scan
# cannot exercise this change at all (both legs render `new DBS$1(s)` and the scan is clean by
# construction). This scan renders each fixture **directory** as one jar, so every family's
# classes resolve against each other — the reading the change is about — and diffs the two legs.
#
#   sh openspec/changes/recover-double-brace-allocation-site/results/scan-corpus-jars.sh <baseline-binary> <change-binary> <out-dir>
#
# Self-tested: both binaries executable, equal capture counts, no empty capture.
set -eu

baseline=$1
change=$2
out=$3
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)

for binary in "$baseline" "$change"; do
    [ -x "$binary" ] || { echo "not executable: $binary" >&2; exit 2; }
done

rm -rf "$out"
mkdir -p "$out/a" "$out/b"

key_of() {
    head=$(printf '%s' "$1" | tr '/$' '__' | cut -c1-100)
    digest=$(printf '%s' "$1" | shasum -a 1 | cut -c1-8)
    printf '%s__%s' "$head" "$digest"
}

# Every directory below the fixture root that holds class files, each zipped whole.
find "$root/tests/fixtures" -type d | sort | while IFS= read -r directory; do
    count=$(find "$directory" -maxdepth 1 -name '*.class' | wc -l | tr -d ' ')
    [ "$count" -gt 0 ] || continue
    relative=${directory#"$root/tests/fixtures/"}
    jar="$out/$(key_of "$relative").jar"
    (cd "$directory" && zip -q -X "$jar" ./*.class)
    find "$directory" -maxdepth 1 -name '*.class' | sort | while IFS= read -r class; do
        name=$(basename "$class" .class)
        subject="$relative::$name.class"
        for leg in a b; do
            if [ "$leg" = a ]; then binary=$baseline; else binary=$change; fi
            status=0
            "$binary" class-source --policy plain-jar --input "$jar" --class "$name" \
                --format text >"$out/$leg/$(key_of "$subject").txt" \
                2>"$out/$leg/$(key_of "$subject").err" || status=$?
            if [ "$status" -ne 0 ] && [ ! -s "$out/$leg/$(key_of "$subject").txt" ]; then
                printf 'unbound (exit %s)\n' "$status" >"$out/$leg/$(key_of "$subject").txt"
            fi
            printf '\n[exit %s]\n' "$status" >>"$out/$leg/$(key_of "$subject").txt"
            printf '# %s\n' "$subject" >>"$out/$leg/$(key_of "$subject").txt"
        done
    done
done

count_a=$(find "$out/a" -name '*.txt' | wc -l | tr -d ' ')
count_b=$(find "$out/b" -name '*.txt' | wc -l | tr -d ' ')
[ "$count_a" = "$count_b" ] || { echo "capture counts differ: $count_a vs $count_b" >&2; exit 3; }
[ "$count_a" -gt 0 ] || { echo "no captures" >&2; exit 4; }
empty=$(find "$out" -name '*.txt' -size 0 | wc -l | tr -d ' ')
[ "$empty" = 0 ] || { echo "$empty empty capture(s)" >&2; exit 5; }

echo "captures: $count_a per leg"
diff -r "$out/a" "$out/b" >"$out/diff.txt" || true
echo "differing captures: $(grep -c '^diff -r' "$out/diff.txt" || true)"
grep '^diff -r' "$out/diff.txt" | grep '\.txt' || true
