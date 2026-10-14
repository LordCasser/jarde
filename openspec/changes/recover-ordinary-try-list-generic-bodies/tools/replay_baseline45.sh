#!/bin/sh
set -eu
TOOL_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_ROOT=$(CDPATH= cd -- "$TOOL_DIR/../../../.." && pwd)
FIXTURE_INPUTS="$PROJECT_ROOT/tests/fixtures/cf11-ordinary-try-list-generic/baseline"
export FIXTURE_INPUTS
exec "$TOOL_DIR/replay_full_class.sh" "$@"
