#!/bin/sh
set -eu

TOOL_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_ROOT=$(CDPATH= cd -- "$TOOL_DIR/../../../.." && pwd)
FIXTURE_INPUTS=${FIXTURE_INPUTS:-"$PROJECT_ROOT/tests/fixtures/cf11-ordinary-try-list-generic/baseline"}

: "${CURRENT_CLI_PATH:?Set the reviewed frozen Jarde CLI path}"
: "${CURRENT_CLI_SHA256:?Set the exact frozen CLI SHA-256}"
: "${CURRENT_BUILD_RECEIPT:?Set the matching build receipt path}"
: "${CURRENT_BUILD_RECEIPT_SHA256:?Set the exact build receipt SHA-256}"
: "${RECEIPT_ACCEPTED_VALUE:?Set the exact accepted receipt status}"
: "${BASELINE_OUT:?Set a fresh output directory that does not exist}"
: "${JDK_MANIFEST:?Set the JDK 8/23 manifest path}"
: "${JDK_MANIFEST_SHA256:?Set the JDK manifest SHA-256}"
: "${JADX_PATH:?Set the reviewed JADX executable path}"
: "${JADX_SHA256:?Set the JADX executable SHA-256}"
: "${JADX_JAR:?Set the reviewed JADX jar path}"
: "${JADX_JAR_SHA256:?Set the JADX jar SHA-256}"
: "${GUARD_PATH:?Set the reviewed process/resource guard path}"
: "${GUARD_SHA256:?Set the guard SHA-256}"
: "${SCANNER_PATH:?Set the reviewed target-size scanner path}"
: "${SCANNER_SHA256:?Set the scanner SHA-256}"
: "${CLASSFILE_PARSER_PATH:?Set the reviewed classfile parser path}"
: "${CLASSFILE_PARSER_SHA256:?Set the parser SHA-256}"

BASELINE_DEBUG_INFO=${BASELINE_DEBUG_INFO:-debug}
RECEIPT_CLI_OBJECT=${RECEIPT_CLI_OBJECT:-cli}
RECEIPT_STATUS_KEY=${RECEIPT_STATUS_KEY:-status}
RECEIPT_MODE_KEY=${RECEIPT_MODE_KEY:-}

exec python3 -B "$TOOL_DIR/collect_full_class.py" \
  --inputs-dir "$FIXTURE_INPUTS" \
  --inputs-manifest "$FIXTURE_INPUTS/manifest.json" \
  --out "$BASELINE_OUT" \
  --javac-debug "$BASELINE_DEBUG_INFO" \
  --cli "$CURRENT_CLI_PATH" \
  --cli-sha256 "$CURRENT_CLI_SHA256" \
  --build-receipt "$CURRENT_BUILD_RECEIPT" \
  --build-receipt-sha256 "$CURRENT_BUILD_RECEIPT_SHA256" \
  --receipt-cli-object "$RECEIPT_CLI_OBJECT" \
  --receipt-status-key "$RECEIPT_STATUS_KEY" \
  --receipt-accepted-value "$RECEIPT_ACCEPTED_VALUE" \
  --receipt-mode-key "$RECEIPT_MODE_KEY" \
  --jdk-manifest "$JDK_MANIFEST" \
  --jdk-manifest-sha256 "$JDK_MANIFEST_SHA256" \
  --jadx "$JADX_PATH" \
  --jadx-sha256 "$JADX_SHA256" \
  --jadx-jar "$JADX_JAR" \
  --jadx-jar-sha256 "$JADX_JAR_SHA256" \
  --guard "$GUARD_PATH" \
  --guard-sha256 "$GUARD_SHA256" \
  --scanner "$SCANNER_PATH" \
  --scanner-sha256 "$SCANNER_SHA256" \
  --classfile-parser "$CLASSFILE_PARSER_PATH" \
  --classfile-parser-sha256 "$CLASSFILE_PARSER_SHA256"
