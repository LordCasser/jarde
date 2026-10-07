#!/bin/sh
# Render one class of one jar with the worktree's own CLI.
#
#   render.sh <jar> <class> [policy]
#
# The class-source text is the presentation; the CLI's bookkeeping planes go to stderr
# (dropped here).  The self-header is asserted before the text is accepted, so a render
# that is not this tool's own presentation can never be counted as one.
set -eu
jar="$1"
class="$2"
policy="${3:-plain-jar}"
cli="${CLI:-./target/debug/jarde-cli}"
text=$("$cli" class-source --input "$jar" --class "$class" --policy "$policy" --format text 2>/dev/null)
case "$text" in
    *"// jarde: presentation of"*) ;;
    *) echo "SELF-TEST FAILED: no jarde presentation header for $class in $jar" >&2; exit 1 ;;
esac
printf '%s\n' "$text"
