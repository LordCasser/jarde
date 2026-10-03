#!/bin/bash
# Corpus two-leg scan for recover-nested-generic-class-headers.
# Usage: scan_corpus.sh <output.sha>   (run leg 1 on a baseline build, leg 2 on the change)
# Transcribes every repo .class file with the release class_source_file example,
# recording per-file: status + sha256 of stdout (or EXIT code on failure).
set -u
OUT="$1"
BIN="$(git rev-parse --show-toplevel)/target/release/examples/class_source_file"
cd "$(git rev-parse --show-toplevel)" || exit 1
: > "$OUT"
COUNT=0
while IFS= read -r class; do
  TEXT="$("$BIN" "$class" 2>/dev/null)"
  CODE=$?
  if [ $CODE -eq 0 ]; then
    SHA=$(printf '%s' "$TEXT" | shasum -a 256 | cut -d' ' -f1)
    echo "OK $SHA $class" >> "$OUT"
  else
    echo "EXIT$CODE - $class" >> "$OUT"
  fi
  COUNT=$((COUNT+1))
done < <(find tests/fixtures fuzz/corpus openspec/evidence -name '*.class' -type f \
  -not -path '*/nested-generic-header-patrol/ngh/*' | sort)
echo "scanned $COUNT classes into $OUT" >&2
