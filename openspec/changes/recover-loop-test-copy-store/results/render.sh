#!/bin/sh
# Render one class of one artifact with this worktree's own CLI.
#
#   render.sh <artifact> <class> <policy>
#
# The class-source text is the presentation; the CLI's bookkeeping planes go to stderr
# (dropped here).  The self-header is asserted before the text is accepted, so a render
# that is not this tool's own presentation can never be counted as one.
set -eu
artifact="$1"
class="$2"
policy="${3:-plain-jar}"
cli="${CLI:-./target/debug/jarde-cli}"
text=$("$cli" class-source --input "$artifact" --class "$class" --policy "$policy" --format text 2>/dev/null)
case "$text" in
    *"// jarde: presentation of"*) ;;
    *) echo "SELF-TEST FAILED: no jarde presentation header for $class in $artifact" >&2; exit 1 ;;
esac
printf '%s\n' "$text"
