#!/bin/bash
# Corpus two-leg scan for recover-assert-statement-sugar (scg precedent).
set -u
OUT="$1"; BIN="$2"; ROOT="$3"
cd "$ROOT" || exit 1
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
  -not -path '*/assert-stmt-patrol/asg/*' | sort)
echo "scanned $COUNT classes into $OUT" >&2
