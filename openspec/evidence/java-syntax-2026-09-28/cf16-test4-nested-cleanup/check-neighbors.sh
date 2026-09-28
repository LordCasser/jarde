#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 2 ]]; then echo 'usage: check-neighbors.sh JARDE_CLI OUTPUT_DIR' >&2; exit 2; fi
HERE="$(cd "$(dirname "$0")" && pwd)"
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
OUT="$2"
PACKAGE=jadx/tests/integration/trycatch
CLASS='TestTryCatchFinally4$TestCls'
mkdir -p "$OUT"
python3 "$HERE/mutate-neighbors.py" "$OUT" > "$OUT/hashes.txt"
javac --release 8 -Xlint:-options -d "$OUT/runner" \
  "$HERE/original/$CLASS.java" "$HERE/TargetRunner.java"
for name in wrong-close-call wrong-delete-effect wrong-catch-type wrong-cleanup-coverage; do
  variant="$OUT/$name"
  mkdir -p "$variant/tmp"
  java -Xverify:all -Djava.io.tmpdir="$variant/tmp" -cp "$variant:$OUT/runner" \
    jadx.tests.integration.trycatch.TargetRunner > "$variant/run.txt"
  "$CLI" class-source --input "$variant/$PACKAGE/$CLASS.class" \
    --class "jadx.tests.integration.trycatch.$CLASS" --policy single-class \
    --release 8 --format text > "$variant/jarde.java.txt" 2> "$variant/jarde.report.txt"
  rg -q 'methods.1.outcome.report.quality = "fallback"' "$variant/jarde.report.txt"
  if rg -q '        } finally \{' "$variant/jarde.java.txt"; then
    echo "$name unexpectedly merged cleanup" >&2; exit 1
  fi
done
if [[ -z "$(find "$OUT/wrong-delete-effect/tmp" -type f -print -quit)" ]]; then
  echo 'the delete-to-exists mutation did not leave its temporary file' >&2; exit 1
fi
if [[ -n "$(find "$OUT/wrong-close-call/tmp" -type f -print -quit)" \
   || -n "$(find "$OUT/wrong-catch-type/tmp" -type f -print -quit)" \
   || -n "$(find "$OUT/wrong-cleanup-coverage/tmp" -type f -print -quit)" ]]; then
  echo 'a control variant unexpectedly left its temporary file' >&2; exit 1
fi
javac --release 8 -Xlint:-options -d "$OUT/wrong-receiver" \
  "$HERE/negative/WrongReceiver.java" "$HERE/negative/WrongReceiverRunner.java"
java -Xverify:all -cp "$OUT/wrong-receiver" \
  jadx.tests.integration.trycatch.WrongReceiverRunner > "$OUT/wrong-receiver/run.txt"
rg -Fq 'body-io|[first.write:1, other.close, file.delete]|IOException:body' \
  "$OUT/wrong-receiver/run.txt"
"$CLI" class-source --input "$OUT/wrong-receiver/$PACKAGE/WrongReceiver.class" \
  --class jadx.tests.integration.trycatch.WrongReceiver --policy single-class \
  --release 8 --format text > "$OUT/wrong-receiver/jarde.java.txt" \
  2> "$OUT/wrong-receiver/jarde.report.txt"
rg -q 'methods.1.outcome.report.quality = "fallback"' \
  "$OUT/wrong-receiver/jarde.report.txt"
printf 'four_fixed_class_mutations_verify_and_refuse=true\nwrong_delete_leaves_file=true\nwrong_receiver_changes_body_exception_order_and_refuses=true\n' \
  > "$OUT/results.txt"
cat "$OUT/results.txt"
