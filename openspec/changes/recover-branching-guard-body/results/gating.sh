#!/bin/sh
# The gating experiment: the baseline (the worktree at HEAD) against this slice's admission, over
# the five guard-family anchor sets plus this slice's own fixture — and, when the certificate-only
# binary is given, the finer column that separates the certificate's admission from the region
# walk's own boundary.
#
#   gating.sh <baseline-cli> <admission-cli> <output-directory> [certificate-only-cli]
#
# Claim this measures:
#   * the admission alone flips the branching anchor (`MLProbe.nestedLocksBranching` on both legs,
#     and the whole of this slice's own fixture) and **nothing else**;
#   * every other render of the five families is byte-identical to the baseline — LK (jar + both
#     legs + its negatives + its probe), IO (jar + both legs + IOMidRead + IONegatives +
#     NestedDepth), the nested-lock family's ML/MLOrder/MLNegatives, the loop-test-copy family and
#     the copy/dup-store family's anchors and controls;
#   * the **certificate-only** column (this slice's `guard.rs` with `region.rs` at HEAD): the
#     certificate claims the shape and the body walk then refuses it, so the anchor's *diagnostic*
#     moves but the member is still refused — the presentation needs the walk's own boundary, which
#     is one statement in `bounded_shared_finally_body` and no new region shape.
#
# The binaries are the same worktree built three times:
#
#   git stash push crates/jarde-java/src/guard.rs crates/jarde-java/src/region.rs
#   cargo build --locked -p jarde-cli && cp target/debug/jarde-cli <baseline-cli>
#   git stash pop
#   cargo build --locked -p jarde-cli && cp target/debug/jarde-cli <admission-cli>
#   git show <feat-commit>^:crates/jarde-java/src/region.rs > /tmp/region-head.rs
#   cp crates/jarde-java/src/region.rs /tmp/region-admission.rs && cp /tmp/region-head.rs crates/jarde-java/src/region.rs
#   cargo build --locked -p jarde-cli && cp target/debug/jarde-cli <certificate-only-cli>
#   cp /tmp/region-admission.rs crates/jarde-java/src/region.rs
#
# The renders go through `render-set.sh`, which asserts every render's own header before it is
# accepted, so a failed render can never be counted as an answer.
set -eu

BASELINE=$1
ADMISSION=$2
OUT=$3
CERTIFICATE_ONLY=${4:-}
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

[ -x "$BASELINE" ] || { echo "no baseline binary at $BASELINE" >&2; exit 2; }
[ -x "$ADMISSION" ] || { echo "no admission binary at $ADMISSION" >&2; exit 2; }

mkdir -p "$OUT"
sh "$HERE/render-set.sh" "$BASELINE" "$OUT/baseline" >/dev/null
sh "$HERE/render-set.sh" "$ADMISSION" "$OUT/admission" >/dev/null
if [ -n "$CERTIFICATE_ONLY" ]; then
    [ -x "$CERTIFICATE_ONLY" ] || { echo "no certificate-only binary at $CERTIFICATE_ONLY" >&2; exit 2; }
    sh "$HERE/render-set.sh" "$CERTIFICATE_ONLY" "$OUT/certificate-only" >/dev/null
fi

{
    echo "# The gating experiment: baseline (HEAD) vs the admission"
    echo "# columns: input | base | admission | verdict"
    echo "input|base|admission|verdict"
} > "$OUT/gating.out"
for file in "$OUT/baseline"/*.java; do
    name=$(basename "$file" .java)
    base=$(shasum -a 256 "$OUT/baseline/$name.java" | cut -d' ' -f1)
    admission=$(shasum -a 256 "$OUT/admission/$name.java" | cut -d' ' -f1)
    verdict="identical"
    [ "$admission" = "$base" ] || verdict="MOVED"
    printf '%s|%s|%s|%s\n' "$name" "$base" "$admission" "$verdict" >> "$OUT/gating.out"
done

if [ -n "$CERTIFICATE_ONLY" ]; then
    {
        echo
        echo "# the finer column: base | certificate-only (guard.rs admitted, region.rs at HEAD) | admission"
        echo "input|base|certificate-only|admission|verdict"
    } > "$OUT/gating-certificate-only.out"
    for file in "$OUT/baseline"/*.java; do
        name=$(basename "$file" .java)
        base=$(shasum -a 256 "$OUT/baseline/$name.java" | cut -d' ' -f1)
        certificate=$(shasum -a 256 "$OUT/certificate-only/$name.java" | cut -d' ' -f1)
        admission=$(shasum -a 256 "$OUT/admission/$name.java" | cut -d' ' -f1)
        verdict="all-identical"
        if [ "$certificate" != "$base" ] && [ "$admission" = "$base" ]; then
            verdict="certificate-only-moved"
        elif [ "$admission" != "$base" ] && [ "$certificate" = "$base" ]; then
            verdict="admission-only-moved"
        elif [ "$admission" != "$base" ] && [ "$certificate" != "$base" ]; then
            if [ "$certificate" = "$admission" ]; then
                verdict="both-moved,same-text"
            else
                verdict="both-moved,different-text"
            fi
        fi
        printf '%s|%s|%s|%s|%s\n' "$name" "$base" "$certificate" "$admission" "$verdict" >> "$OUT/gating-certificate-only.out"
    done
    cat "$OUT/gating-certificate-only.out"
fi
cat "$OUT/gating.out"
