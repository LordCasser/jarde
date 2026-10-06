#!/bin/sh
# Builds the frozen fixtures of this change into their two javac legs. The compile-only
# definitions under `api1/` (the shape each class was really compiled against) are staged in
# /tmp and never shipped; the contradicting definitions under `api2/` are shipped as the leg's
# `ArityApi.class` / `NB$Twin.class`, which is what makes those two cells refusal cells.
#
#   sh openspec/changes/recover-parameterized-interface-headers/results/03-fixtures/build-fixtures.sh
set -eu

J8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home
ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a10fff-5ca8-7423-97ff-8582a1fb7e61
WORK=/tmp/pih-fixtures
IFACE="$ROOT/tests/fixtures/p3-interface-header-projection"
PARENT="$ROOT/tests/fixtures/p3-nested-parent-projection"
rm -rf "$WORK"
mkdir -p "$WORK/iface-api1-8" "$WORK/iface-api2-8" "$WORK/parent-api1-8" "$WORK/parent-api2-8"
mkdir -p "$WORK/iface-api1-23" "$WORK/iface-api2-23" "$WORK/parent-api1-23" "$WORK/parent-api2-23"
mkdir -p "$IFACE/v8" "$IFACE/v8-javac8" "$PARENT/v8" "$PARENT/v8-javac8"
rm -f "$IFACE"/v8/*.class "$IFACE"/v8-javac8/*.class "$PARENT"/v8/*.class "$PARENT"/v8-javac8/*.class

# ---- stage 1: the compile-only definitions ------------------------------------------------
"$J8/bin/javac" -d "$WORK/iface-api1-8" "$IFACE/api1/MissingApi.java" "$IFACE/api1/ArityApi.java"
javac --release 8 -Xlint:-options -d "$WORK/iface-api1-23" "$IFACE/api1/MissingApi.java" "$IFACE/api1/ArityApi.java"
"$J8/bin/javac" -d "$WORK/parent-api1-8" "$PARENT/api1/NB\$Twin.java"
javac --release 8 -Xlint:-options -d "$WORK/parent-api1-23" "$PARENT/api1/NB\$Twin.java"

# ---- stage 2: the contradicting definitions the environments ship --------------------------
"$J8/bin/javac" -d "$WORK/iface-api2-8" "$IFACE/api2/ArityApi.java"
javac --release 8 -Xlint:-options -d "$WORK/iface-api2-23" "$IFACE/api2/ArityApi.java"
"$J8/bin/javac" -d "$WORK/parent-api2-8" "$PARENT/api2/NB\$Twin.java"
javac --release 8 -Xlint:-options -d "$WORK/parent-api2-23" "$PARENT/api2/NB\$Twin.java"

# ---- stage 3: the fixture classes, per leg -------------------------------------------------
compile_leg() {
    # `mode` is `8` (Corretto 1.8.0_432) or `23` (host javac, `--release 8`).
    mode=$1
    out=$2
    api1=$3
    api2=$4
    base=$5
    shift 5
    if [ "$mode" = 8 ]; then
        "$J8/bin/javac" -classpath "$api1:$base" -d "$out" "$@"
    else
        javac --release 8 -Xlint:-options -classpath "$api1:$base" -d "$out" "$@"
    fi
    cp "$api2"/*.class "$out/"
}

compile_leg 8 "$IFACE/v8-javac8" "$WORK/iface-api1-8" "$WORK/iface-api2-8" "" \
    "$IFACE/IfaceImpl.java" "$IFACE/MultiIface.java" "$IFACE/ErasedCall.java" \
    "$IFACE/Unresolved.java" "$IFACE/Arity.java" "$IFACE/TypeUse.java" "$IFACE/Mark.java"
compile_leg 23 "$IFACE/v8" "$WORK/iface-api1-23" "$WORK/iface-api2-23" "" \
    "$IFACE/IfaceImpl.java" "$IFACE/MultiIface.java" "$IFACE/ErasedCall.java" \
    "$IFACE/Unresolved.java" "$IFACE/Arity.java" "$IFACE/TypeUse.java" "$IFACE/Mark.java"

# The `$`-named parents must be **class files** on the classpath, not sources in the same javac
# run: inside one run javac's own symbol table knows `NB.Box`, and the flat name `NB$Box` only
# resolves from a class file (the same rule the recovered-source legs of the tests follow).
for mode in 8 23; do
    mkdir -p "$WORK/parent-base-$mode"
    if [ "$mode" = 8 ]; then
        "$J8/bin/javac" -d "$WORK/parent-base-8" "$PARENT/NB.java" "$PARENT/MO.java"
    else
        javac --release 8 -Xlint:-options -d "$WORK/parent-base-23" "$PARENT/NB.java" "$PARENT/MO.java"
    fi
done

# One file per javac run: a flat (`$`-containing) name resolves from a class file only once per
# run (the first file to load it wins; a second file naming it reports "cannot find symbol"), so
# the cells that share the `NB$` parents are compiled apart. The recovered-source legs of the
# tests keep one reference per flat name per compile for the same reason.
for mode in 8 23; do
    for src in NestedExtends BareBox Multiseg ArityExtends; do
        if [ "$mode" = 8 ]; then
            "$J8/bin/javac" -classpath "$WORK/parent-api1-8:$WORK/parent-base-8" \
                -d "$PARENT/v8-javac8" "$PARENT/$src.java"
        else
            javac --release 8 -Xlint:-options -classpath "$WORK/parent-api1-23:$WORK/parent-base-23" \
                -d "$PARENT/v8" "$PARENT/$src.java"
        fi
    done
    case $mode in
        8) cp "$WORK/parent-api2-8"/*.class "$PARENT/v8-javac8/";;
        23) cp "$WORK/parent-api2-23"/*.class "$PARENT/v8/";;
    esac
done
# The parent base classes ship too: the environments resolve them by their flat names.
for legdir in "$PARENT/v8-javac8:8" "$PARENT/v8:23"; do
    leg=${legdir%%:*}
    mode=${legdir##*:}
    cp "$WORK/parent-base-$mode"/*.class "$leg/"
done

echo "== interface leg =="
ls "$IFACE/v8" "$IFACE/v8-javac8"
echo "== parent leg =="
ls "$PARENT/v8" "$PARENT/v8-javac8"

echo "== the header facts the fixtures carry (javac 8 leg) =="
"$J8/bin/javap" -p -v -classpath "$IFACE/v8-javac8" IfaceImpl | grep -E "^public class|Signature:|ACC_BRIDGE" -A1 | head -8
"$J8/bin/javap" -p -v -classpath "$IFACE/v8-javac8" Unresolved | grep -E "^public class|Signature:" | head -4
"$J8/bin/javap" -p -v -classpath "$IFACE/v8-javac8" Arity | grep -E "^public class|Signature:" | head -4
"$J8/bin/javap" -p -v -classpath "$PARENT/v8-javac8" NestedExtends | grep -E "^public class|Signature:" | head -4
"$J8/bin/javap" -p -v -classpath "$PARENT/v8-javac8" ArityExtends | grep -E "^public class|Signature:" | head -4
"$J8/bin/javap" -p -v -classpath "$PARENT/v8-javac8" BareBox | grep -E "^public class|Signature:" | head -3
echo "FIXTURES BUILT"
