#!/bin/sh
# 1.4 replay of the Q2 discriminating experiment (root's `/tmp/rq2b` + `/tmp/rq2d`), with the
# deep-nested `D$Mid$Leaf<Integer>` cell. The question: is a pool-spelled (`$`-containing) name
# legal in a class header *with type arguments*, and does it compile to the same product as the
# source-spelled (`D.Base`) form?
#
# Self-test (mandatory): the harness must show a known *failure* as a failure, so a missing name
# (`D$NotThere<String>`) is compiled and its nonzero exit is required. A harness that cannot fail
# proves nothing.
#
# Design: each variant pair declares the SAME class under the SAME name in two directories that
# differ only in the parent spelling, so the products are byte-comparable with `cmp`. The first
# run (commands with every variant in ONE javac invocation) is recorded as an invalid round: the
# raw and parameterized pool spellings of the same flat name do not resolve inside one javac run
# (the first file to load the flat name wins; the second reports "cannot find symbol"). Each
# variant is therefore compiled alone.
set -eu

J8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home
WORK=/tmp/pih-q2-replay
rm -rf "$WORK"
mkdir -p "$WORK/base8" "$WORK/base23" "$WORK/pool8" "$WORK/source8" "$WORK/pool23" "$WORK/source23"

cat >"$WORK/D.java" <<'EOF'
public class D {
    public static class Base<T> {
        public T value;
    }
    public static class Mid {
        public static class Leaf<U> {
            public U leaf;
        }
    }
}
EOF
"$J8/bin/javac" -d "$WORK/base8" "$WORK/D.java"
javac --release 8 -Xlint:-options -d "$WORK/base23" "$WORK/D.java"

write_variants() {
    dir=$1
    parent_raw=$2
    parent_par=$3
    parent_deep=$4
    cat >"$dir/Same.java" <<EOF
public class Same extends $parent_par {
    public String value;
    public static void main(String[] a) { System.out.println("same"); }
}
EOF
    cat >"$dir/SameRaw.java" <<EOF
public class SameRaw extends $parent_raw {
    public Object value;
}
EOF
    cat >"$dir/SameDeep.java" <<EOF
public class SameDeep extends $parent_deep {
    public Integer value;
    public static void main(String[] a) { System.out.println("deep"); }
}
EOF
}

write_variants "$WORK/pool8" 'D$Base' 'D$Base<String>' 'D$Mid$Leaf<Integer>'
write_variants "$WORK/source8" 'D.Base' 'D.Base<String>' 'D.Mid.Leaf<Integer>'
write_variants "$WORK/pool23" 'D$Base' 'D$Base<String>' 'D$Mid$Leaf<Integer>'
write_variants "$WORK/source23" 'D.Base' 'D.Base<String>' 'D.Mid.Leaf<Integer>'

echo "== javac 8 (Corretto 1.8.0_432) =="
for variant in Same SameRaw SameDeep; do
    "$J8/bin/javac" -classpath "$WORK/base8" -d "$WORK/pool8" "$WORK/pool8/$variant.java"
    echo "pool   $variant javac8 exit=0"
    "$J8/bin/javac" -classpath "$WORK/base8" -d "$WORK/source8" "$WORK/source8/$variant.java"
    echo "source $variant javac8 exit=0"
done

echo "== javac 23.0.1 --release 8 =="
for variant in Same SameRaw SameDeep; do
    javac --release 8 -Xlint:-options -classpath "$WORK/base23" -d "$WORK/pool23" "$WORK/pool23/$variant.java"
    echo "pool   $variant javac23 exit=0"
    javac --release 8 -Xlint:-options -classpath "$WORK/base23" -d "$WORK/source23" "$WORK/source23/$variant.java"
    echo "source $variant javac23 exit=0"
done

echo "== products: pool spelling vs source spelling, byte for byte =="
for leg in 8 23; do
    for variant in Same SameRaw SameDeep; do
        if cmp -s "$WORK/pool$leg/$variant.class" "$WORK/source$leg/$variant.class"; then
            echo "javac$leg $variant: IDENTICAL"
        else
            echo "javac$leg $variant: DIFFERENT"
            exit 1
        fi
    done
done

echo "== self-test: a missing flat name must fail =="
mkdir -p "$WORK/negative"
cat >"$WORK/negative/Missing.java" <<'EOF'
public class Missing extends D$NotThere<String> { }
EOF
if "$J8/bin/javac" -classpath "$WORK/base8" -d "$WORK/negative" "$WORK/negative/Missing.java" 2>"$WORK/negative/javac8.err"; then
    echo "SELF-TEST FAILED: the missing name compiled"
    exit 1
fi
if javac --release 8 -Xlint:-options -classpath "$WORK/base23" -d "$WORK/negative" "$WORK/negative/Missing.java" 2>"$WORK/negative/javac23.err"; then
    echo "SELF-TEST FAILED: the missing name compiled (javac 23)"
    exit 1
fi
echo "SELF-TEST OK: missing name fails on both legs"
echo "== the header facts the products carry =="
"$J8/bin/javap" -p -v -classpath "$WORK/pool8" Same | sed -n '1,4p'
"$J8/bin/javap" -p -v -classpath "$WORK/pool8" Same | grep -E "Signature:|InnerClasses" -A2 | head -8
"$J8/bin/javap" -p -v -classpath "$WORK/pool8" SameDeep | sed -n '1,4p'
echo "Q2 REPLAY OK"
