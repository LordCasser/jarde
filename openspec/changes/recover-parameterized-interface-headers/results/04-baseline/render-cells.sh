#!/bin/sh
# Renders every frozen cell of this change through one `jarde-cli` binary and prints, per cell,
# the declaration line and each bridge proof's state. Run it once with the pre-change binary
# (`/tmp/pih-bin/jarde-cli-base`) and once with the change's own build; the two transcripts are
# the "before" and "after" records of tasks 1.2/1.3/1.5.
#
#   sh .../04-baseline/render-cells.sh /tmp/pih-bin/jarde-cli-base > before.txt
#
# Self-tests (mandatory): a known *positive* (the text render carries the jarde self-header) and
# a known *negative* (a jar whose class name does not exist must fail) run before any cell is
# measured, so a harness that cannot fail never contributes a number.
set -eu

BIN=$1
ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a10fff-5ca8-7423-97ff-8582a1fb7e61
IFACE="$ROOT/tests/fixtures/p3-interface-header-projection"
PARENT="$ROOT/tests/fixtures/p3-nested-parent-projection"
BR="$ROOT/tests/fixtures/p3-bridge-projection/br-family/v8"
SPEC="$ROOT/tests/fixtures/p3-bridge-projection/bridge-superclass-precondition/v8"
WORK=/tmp/pih-cells
rm -rf "$WORK"
mkdir -p "$WORK"

pack() {
    jar=$1
    shift
    rm -f "$jar"
    zip -j -q "$jar" "$@"
}

cell() {
    label=$1
    jar=$2
    name=$3
    out="$WORK/$label.json"
    "$BIN" class-source --policy plain-jar --input "$jar" --class "$name" --format json --output "$out" >"$WORK/$label.stdout" 2>"$WORK/$label.stderr" || true
    if [ ! -s "$out" ]; then
        echo "== $label: NO OUTPUT (see $WORK/$label.stderr)"
        return
    fi
    python3 - "$label" "$out" <<'PY'
import json, sys
label, path = sys.argv[1], sys.argv[2]
report = json.load(open(path))
declaration = report.get("declaration", {}).get("declaration", "<none>")
print(f"== {label}: {declaration}")
generic = report.get("declaration", {}).get("generic_refusal")
if generic:
    print(f"   header refusal: {generic}")
for proof in report.get("bridge_proofs", []):
    member = proof.get("member", {})
    descriptor = bytes(member.get("descriptor", [])).decode("utf-8", "replace")
    name = member.get("name")
    name = bytes(name).decode("utf-8", "replace") if isinstance(name, list) else name
    refusal = proof.get("refusal") or "-"
    print(f"   bridge {name}{descriptor}: admitted={proof.get('admitted')} projected={proof.get('projected')} refusal={refusal}")
PY
}

# ---- self-tests ------------------------------------------------------------------------------
pack "$WORK/self.jar" "$BR/BR\$Impl.class" "$BR/BR\$Box.class" "$BR/BR\$Node.class"
"$BIN" class-source --policy plain-jar --input "$WORK/self.jar" --class 'BR$Impl' --format text --output "$WORK/self.txt" >/dev/null 2>&1 || true
if ! grep -q '^// jarde: presentation of' "$WORK/self.txt"; then
    echo "SELF-TEST FAILED: the text render carries no jarde self-header"
    exit 1
fi
if "$BIN" class-source --policy plain-jar --input "$WORK/self.jar" --class NoSuchClass --format json --output "$WORK/negative.json" >/dev/null 2>&1; then
    echo "SELF-TEST FAILED: a name that does not exist was answered as a success"
    exit 1
fi
if grep -q '^// jarde: presentation of' "$WORK/negative.json" 2>/dev/null; then
    echo "SELF-TEST FAILED: the negative render carries a source header"
    exit 1
fi
echo "SELF-TEST OK: self-header present on a real render; a missing name is not answered as source"
echo "binary: $BIN"

# ---- the cells -------------------------------------------------------------------------------
pack "$WORK/iface-single.jar" "$IFACE/v8/IfaceImpl.class"
cell iface-single "$WORK/iface-single.jar" IfaceImpl

pack "$WORK/iface-multi.jar" "$IFACE/v8/MultiIface.class"
cell iface-multi "$WORK/iface-multi.jar" MultiIface

pack "$WORK/iface-erased.jar" "$IFACE/v8/ErasedCall.class"
cell iface-erased "$WORK/iface-erased.jar" ErasedCall

pack "$WORK/iface-unresolved.jar" "$IFACE/v8/Unresolved.class"
cell iface-unresolved "$WORK/iface-unresolved.jar" Unresolved

pack "$WORK/iface-arity.jar" "$IFACE/v8/Arity.class" "$IFACE/v8/ArityApi.class"
cell iface-arity "$WORK/iface-arity.jar" Arity

pack "$WORK/iface-typeuse.jar" "$IFACE/v8/TypeUse.class" "$IFACE/v8/Mark.class"
cell iface-typeuse "$WORK/iface-typeuse.jar" TypeUse

pack "$WORK/br-family.jar" "$BR/BR.class" "$BR/BR\$Node.class" "$BR/BR\$Box.class" "$BR/BR\$Base.class" "$BR/BR\$StrBox.class" "$BR/BR\$Impl.class"
cell br-impl "$WORK/br-family.jar" 'BR$Impl'
cell br-strbox "$WORK/br-family.jar" 'BR$StrBox'

pack "$WORK/parent-spec.jar" "$SPEC/Outer.class" "$SPEC/Outer\$Box.class" "$SPEC/Spec.class" "$SPEC/Drv.class"
cell parent-spec "$WORK/parent-spec.jar" Spec

pack "$WORK/parent-criteria.jar" "$PARENT/v8/NB.class" "$PARENT/v8/NB\$Root.class" "$PARENT/v8/NB\$Box.class" "$PARENT/v8/NestedExtends.class" "$PARENT/v8/BareBox.class"
cell parent-criteria-nested "$WORK/parent-criteria.jar" NestedExtends
cell parent-criteria-bare "$WORK/parent-criteria.jar" BareBox

pack "$WORK/parent-arity.jar" "$PARENT/v8/NB\$Twin.class" "$PARENT/v8/ArityExtends.class"
cell parent-arity "$WORK/parent-arity.jar" ArityExtends

pack "$WORK/parent-multiseg.jar" "$PARENT/v8/MO.class" "$PARENT/v8/MO\$Mid.class" "$PARENT/v8/Multiseg.class"
cell parent-multiseg "$WORK/parent-multiseg.jar" Multiseg
