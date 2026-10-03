#!/bin/bash
# Standalone `.class` corpus leg: the repo's own `.class` files outside jars, transcribed by the
# `class_source_file` example. Usage: scan_classes.sh <example-binary> <out>
set -u
BIN="$1"; OUT="$2"
cd "$(git rev-parse --show-toplevel)" || exit 1
: > "$OUT"
COUNT=0
while IFS= read -r class; do
  TEXT="$("$BIN" "$class" 2>/dev/null)"
  CODE=$?
  if [ $CODE -eq 0 ]; then
    printf 'OK %s %s\n' "$(printf '%s' "$TEXT" | shasum -a 256 | cut -d' ' -f1)" "$class" >> "$OUT"
  else
    printf 'EXIT%s - %s\n' "$CODE" "$class" >> "$OUT"
  fi
  COUNT=$((COUNT+1))
done < <(find tests/fixtures fuzz/corpus openspec/evidence -name '*.class' -type f | sort)
echo "scanned $COUNT class files into $OUT" >&2
