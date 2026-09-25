#!/usr/bin/env bash
set -euo pipefail

EVIDENCE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$EVIDENCE_DIR/../../../.." && pwd)"
BASE_EVIDENCE="$REPO_ROOT/openspec/evidence/java-syntax-2026-09-24/foreach-cache-declarations"
RUN_ROOT="$(mktemp -d /tmp/jarde-slot-reuse-evidence.XXXXXX)"
trap 'rm -rf "$RUN_ROOT"' EXIT
export CARGO_TARGET_DIR="$RUN_ROOT/cargo-target"
export CARGO_NET_OFFLINE=true

for tool in javac javap java jadx cargo python3; do
    command -v "$tool" >/dev/null || { echo "missing required tool: $tool" >&2; exit 2; }
done
javac --version
jadx --version

mkdir -p "$RUN_ROOT/base-g" "$RUN_ROOT/base-none" "$RUN_ROOT/boundaries-g" "$RUN_ROOT/boundaries-none"
javac --release 8 -g -Xlint:-options -d "$RUN_ROOT/base-g" \
    "$BASE_EVIDENCE/ReuseAfterForEach.java" "$BASE_EVIDENCE/ReuseAfterForEachRunner.java"
javac --release 8 -g:none -Xlint:-options -d "$RUN_ROOT/base-none" \
    "$BASE_EVIDENCE/ReuseAfterForEach.java" "$BASE_EVIDENCE/ReuseAfterForEachRunner.java"
javac --release 8 -g -Xlint:-options -d "$RUN_ROOT/boundaries-g" \
    "$EVIDENCE_DIR/fixtures/SlotReuseBoundaries.java" "$EVIDENCE_DIR/fixtures/SlotReuseBoundariesRunner.java"
javac --release 8 -g:none -Xlint:-options -d "$RUN_ROOT/boundaries-none" \
    "$EVIDENCE_DIR/fixtures/SlotReuseBoundaries.java" "$EVIDENCE_DIR/fixtures/SlotReuseBoundariesRunner.java"

for mode in g none; do
    if [ "$mode" = g ]; then class_dir="$RUN_ROOT/base-g"; else class_dir="$RUN_ROOT/base-none"; fi
    echo "=== original ReuseAfterForEach -$mode ==="
    (cd "$class_dir" && java -Xverify:all ReuseAfterForEachRunner) > "$RUN_ROOT/original-$mode.txt"
    cat "$RUN_ROOT/original-$mode.txt"
    javap -classpath "$class_dir" -c -l -v ReuseAfterForEach > "$RUN_ROOT/reuse-$mode-javap.txt"
done

javap -classpath "$RUN_ROOT/boundaries-g" -c -l -v SlotReuseBoundaries > "$RUN_ROOT/boundaries-g-javap.txt"
javap -classpath "$RUN_ROOT/boundaries-none" -c -l -v SlotReuseBoundaries > "$RUN_ROOT/boundaries-none-javap.txt"
echo "=== boundary fixture -g ==="
(cd "$RUN_ROOT/boundaries-g" && java -Xverify:all SlotReuseBoundariesRunner) > "$RUN_ROOT/boundaries-g-runtime.txt"
cat "$RUN_ROOT/boundaries-g-runtime.txt"
echo "=== boundary fixture -g:none ==="
(cd "$RUN_ROOT/boundaries-none" && java -Xverify:all SlotReuseBoundariesRunner) > "$RUN_ROOT/boundaries-none-runtime.txt"
cat "$RUN_ROOT/boundaries-none-runtime.txt"
diff -u "$RUN_ROOT/boundaries-g-runtime.txt" "$RUN_ROOT/boundaries-none-runtime.txt"

cargo build --manifest-path "$REPO_ROOT/Cargo.toml" -p jarde-cli --bin jarde-cli
CLI="$CARGO_TARGET_DIR/debug/jarde-cli"
"$CLI" class-source --input "$RUN_ROOT/base-g/ReuseAfterForEach.class" --class ReuseAfterForEach \
    --policy single-class --release 8 --format json --evidence all > "$RUN_ROOT/reuse-all.json"
python3 - "$RUN_ROOT/reuse-all.json" "$RUN_ROOT/reuse-region-order.json" <<'PY_FILTER'
import json, sys
report = json.load(open(sys.argv[1]))
for method in report.get("methods", []):
    identity = method.get("item", {}).get("identity", {})
    if identity.get("name") == list(b"sumThenReuse"):
        result = method.get("outcome", {}).get("report", {})
        json.dump({"method": result.get("method"), "regions": result.get("regions"),
                   "rules": result.get("rules")}, open(sys.argv[2], "w"), indent=2)
        break
else:
    raise SystemExit("sumThenReuse evidence is missing")
PY_FILTER
mkdir -p "$RUN_ROOT/ssa-dump/src"
cat > "$RUN_ROOT/ssa-dump/Cargo.toml" <<EOF_MANIFEST
[package]
name = "slot-ssa-dump"
version = "0.0.0"
edition = "2024"

[dependencies]
jarde-jvm = { path = "$REPO_ROOT/crates/jarde-jvm" }
jarde-reader = { path = "$REPO_ROOT/crates/jarde-reader" }
blake3 = "=1.8.7"
EOF_MANIFEST
cp "$EVIDENCE_DIR/tools/ssa_dump.rs" "$RUN_ROOT/ssa-dump/src/main.rs"
ssa() {
    cargo run --quiet --manifest-path "$RUN_ROOT/ssa-dump/Cargo.toml" -- "$@"
}
ssa "$RUN_ROOT/base-g/ReuseAfterForEach.class" sumThenReuse '([I)I' > "$RUN_ROOT/reuse-g-ssa.txt"
ssa "$RUN_ROOT/base-none/ReuseAfterForEach.class" sumThenReuse '([I)I' > "$RUN_ROOT/reuse-none-ssa.txt"
ssa "$RUN_ROOT/boundaries-g/SlotReuseBoundaries.class" sameType '(I)I' > "$RUN_ROOT/sameType-ssa.txt"
ssa "$RUN_ROOT/boundaries-g/SlotReuseBoundaries.class" exclusiveBranch '(Z)I' > "$RUN_ROOT/exclusiveBranch-ssa.txt"
ssa "$RUN_ROOT/boundaries-g/SlotReuseBoundaries.class" loopBodyReuse '(I)I' > "$RUN_ROOT/loopBodyReuse-ssa.txt"
ssa "$RUN_ROOT/boundaries-g/SlotReuseBoundaries.class" handlerReuse '(Z)I' > "$RUN_ROOT/handlerReuse-ssa.txt"
ssa "$RUN_ROOT/boundaries-g/SlotReuseBoundaries.class" category2Adjacent '()J' > "$RUN_ROOT/category2Adjacent-ssa.txt"
ssa "$REPO_ROOT/tests/fixtures/p3-try-local/v9/Held.class" use '(Ljava/io/Reader;)I' 9 > "$RUN_ROOT/Held-use-ssa.txt"
for mode in g none; do
    if [ "$mode" = g ]; then class_dir="$RUN_ROOT/boundaries-g"; else class_dir="$RUN_ROOT/boundaries-none"; fi
    "$CLI" class-source --input "$class_dir/SlotReuseBoundaries.class" --class SlotReuseBoundaries \
        --policy single-class --release 8 --format json --evidence essential \
        > "$RUN_ROOT/boundaries-jarde-$mode.json"
done
python3 "$EVIDENCE_DIR/tools/verify_isolated_methods.py" "$RUN_ROOT" "$EVIDENCE_DIR"

for mode in g none; do
    if [ "$mode" = g ]; then class_dir="$RUN_ROOT/base-g"; evidence_mode=g; else class_dir="$RUN_ROOT/base-none"; evidence_mode=no-debug; fi
    mkdir -p "$RUN_ROOT/jarde-$mode" "$RUN_ROOT/jadx-$mode" "$RUN_ROOT/jarde-classes-$mode" "$RUN_ROOT/jadx-classes-$mode"
    "$CLI" class-source --input "$class_dir/ReuseAfterForEach.class" --class ReuseAfterForEach \
        --policy single-class --release 8 --format text --evidence essential \
        --output "$RUN_ROOT/jarde-$mode/ReuseAfterForEach.java" 2> "$RUN_ROOT/jarde-$mode/planes.txt"
    javac --release 8 -g:none -Xlint:-options -d "$RUN_ROOT/jarde-classes-$mode" \
        "$RUN_ROOT/jarde-$mode/ReuseAfterForEach.java" "$BASE_EVIDENCE/ReuseAfterForEachRunner.java"
    (cd "$RUN_ROOT/jarde-classes-$mode" && java -Xverify:all ReuseAfterForEachRunner) > "$RUN_ROOT/jarde-$mode/runtime.txt"
    diff -u "$RUN_ROOT/original-$mode.txt" "$RUN_ROOT/jarde-$mode/runtime.txt"

    jadx --no-res --single-class ReuseAfterForEach -d "$RUN_ROOT/jadx-$mode" "$class_dir/ReuseAfterForEach.class" \
        > "$RUN_ROOT/jadx-$mode/stdout.txt" 2> "$RUN_ROOT/jadx-$mode/stderr.txt"
    find "$RUN_ROOT/jadx-$mode" -name ReuseAfterForEach.java -exec cp {} "$RUN_ROOT/jadx-$mode/ReuseAfterForEach.java" \;
    { printf 'package defpackage;\n'; cat "$BASE_EVIDENCE/ReuseAfterForEachRunner.java"; } > "$RUN_ROOT/jadx-$mode/ReuseAfterForEachRunner.java"
    javac --release 8 -g:none -Xlint:-options -d "$RUN_ROOT/jadx-classes-$mode" \
        "$RUN_ROOT/jadx-$mode/ReuseAfterForEach.java" "$RUN_ROOT/jadx-$mode/ReuseAfterForEachRunner.java"
    (cd "$RUN_ROOT/jadx-classes-$mode" && java -Xverify:all defpackage.ReuseAfterForEachRunner) \
        > "$RUN_ROOT/jadx-$mode/runtime.txt"
    diff -u "$RUN_ROOT/original-$mode.txt" "$RUN_ROOT/jadx-$mode/runtime.txt"

    if javac --release 8 -g:none -Xlint:-options -d "$RUN_ROOT/baseline-classes-$mode" \
        "$EVIDENCE_DIR/three-way/baseline-jarde/$evidence_mode/ReuseAfterForEach.java" \
        "$BASE_EVIDENCE/ReuseAfterForEachRunner.java" \
        > "$RUN_ROOT/baseline-jarde-$mode.stdout" 2> "$RUN_ROOT/baseline-jarde-$mode.stderr"; then
        echo "unexpected baseline Jarde compile success (-$mode)" >&2
        exit 1
    else
        echo "=== frozen pre-fix Jarde baseline refuses javac (-$mode) ==="
        cat "$RUN_ROOT/baseline-jarde-$mode.stderr"
    fi
    echo "=== current Jarde -$mode ==="
    sha256sum "$RUN_ROOT/jarde-$mode/ReuseAfterForEach.java"
    cat "$RUN_ROOT/jarde-$mode/runtime.txt"
done

cat "$RUN_ROOT/boundary-isolated-results.txt"
printf '=== private Cargo target is removed on exit: %s ===\n' "$CARGO_TARGET_DIR"
