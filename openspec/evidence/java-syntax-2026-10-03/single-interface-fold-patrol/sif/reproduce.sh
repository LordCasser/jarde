#!/bin/bash
# Reproduce the single-static-interface-fold patrol evidence.
#
# Usage: reproduce.sh <jarde-cli-binary> [workdir]
#   jarde-cli-binary: a built `jarde-cli` (e.g. target/debug/jarde-cli at the commit under test)
#   workdir:          scratch dir for compiled fixtures (default: /tmp/sif-repro)
#
# Prints, for each fixture, whether the root's source unit carries a nested `static class`/
# `static interface` declaration — the fold outcome the patrol reports. The control pair is
# Z3 (single static interface child, no lambda -> FOLD) vs Z6 (same child, lambda root ->
# no-fold); Y1M flips only the child kind against Y1 and does not fold either.
set -u
BIN="$1"
WORK="${2:-/tmp/sif-repro}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git rev-parse --show-toplevel)"
rm -rf "$WORK"; mkdir -p "$WORK"

folded_path() { # jar class
  local n
  n="$("$BIN" class-source --input "$1" --class "$2" 2>/dev/null \
       | grep -cE 'static class |static interface ')"
  [ "$n" -gt 0 ] && echo FOLD || echo no-fold
}

build() { # name
  mkdir -p "$WORK/$1"
  javac --release 8 -d "$WORK/$1" "$HERE/fixtures/$1.java" 2>/dev/null \
    || { echo "$1 JAVAC_FAIL"; return 1; }
  (cd "$WORK/$1" && jar cf "$WORK/$1.jar" .)
}

printf '%-7s %-6s %s\n' FIXTURE CHILD NESTED
for spec in Z3:iface Q1:iface Q3:class Q2:iface Q4:class Z6:iface Y1M:class Y1X:iface WCallI:iface WCallC:class; do
  name="${spec%%:*}"; kind="${spec##*:}"
  build "$name" >/dev/null || continue
  printf '%-7s %-6s %s\n' "$name" "$kind" "$(folded_path "$WORK/$name.jar" "$name")"
done

echo
echo '--- frozen Y1 fixture ---'
Y1JAR="$ROOT/openspec/evidence/java-syntax-2026-10-03/single-interface-fold-patrol/fixture/fam.jar"
printf 'Y1 nested=%s\n' "$(folded_path "$Y1JAR" Y1)"