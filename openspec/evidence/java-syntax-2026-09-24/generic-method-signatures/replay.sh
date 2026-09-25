#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-24/generic-method-signatures"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/jarde-generic-signatures.XXXXXX")
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

mkdir -p "$TMP/subject"
javac --release 8 -g:none -d "$TMP/subject" "$EVIDENCE/GenericMethodProbe.java"
cp "$EVIDENCE/GenericMethodRunner.java" "$TMP/subject/"
javac --release 8 -g:none -cp "$TMP/subject" -d "$TMP/subject" "$TMP/subject/GenericMethodRunner.java"
printf 'subject sha256: '
shasum -a 256 "$TMP/subject/GenericMethodProbe.class" | awk '{print $1}'
printf 'subject independent compile: '
java -Xverify:all -cp "$TMP/subject" GenericMethodRunner | paste -sd '|' -
javap -v -p -classpath "$TMP/subject" GenericMethodProbe > "$TMP/subject.javap"
rg 'descriptor:|Signature:' "$TMP/subject.javap"

jadx -d "$TMP/jadx" "$TMP/subject/GenericMethodProbe.class" >/dev/null
"$ROOT/target/debug/jarde-cli" class-source --input "$TMP/subject/GenericMethodProbe.class" --class GenericMethodProbe --policy single-class --format text > "$TMP/jarde.java" 2> "$TMP/jarde.stderr"

compile_source() {
    label=$1
    source=$2
    package_name=$3
    output="$TMP/$label"
    mkdir -p "$output/src" "$output/classes"
    if [ -n "$package_name" ]; then
        mkdir -p "$output/src/$package_name"
        cp "$source" "$output/src/$package_name/GenericMethodProbe.java"
        cat > "$output/src/$package_name/GenericMethodRunner.java" <<'EOF'
package defpackage;
public class GenericMethodRunner {
    public static void main(String[] args) throws Exception {
        System.out.println(GenericMethodProbe.choose(Integer.valueOf(3), Integer.valueOf(4), true));
        System.out.println(GenericMethodProbe.class.getDeclaredMethod("choose", Number.class, Number.class, boolean.class).getTypeParameters().length);
    }
}
EOF
        javac --release 8 -g:none -d "$output/classes" "$output/src/$package_name/GenericMethodProbe.java" "$output/src/$package_name/GenericMethodRunner.java"
        printf '%s: ' "$label"
        java -Xverify:all -cp "$output/classes" "$package_name.GenericMethodRunner" | paste -sd '|' -
    else
        cp "$source" "$output/src/GenericMethodProbe.java"
        cp "$EVIDENCE/GenericMethodRunner.java" "$output/src/GenericMethodRunner.java"
        javac --release 8 -g:none -d "$output/classes" "$output/src/GenericMethodProbe.java" "$output/src/GenericMethodRunner.java"
        printf '%s: ' "$label"
        java -Xverify:all -cp "$output/classes" GenericMethodRunner | paste -sd '|' -
    fi
    javap -v -p -classpath "$output/classes" "${package_name:+$package_name.}GenericMethodProbe" | rg 'descriptor:|Signature:'
}

compile_source original "$EVIDENCE/GenericMethodProbe.java" ""
compile_source jadx "$TMP/jadx/sources/defpackage/GenericMethodProbe.java" defpackage
compile_source jarde "$TMP/jarde.java" ""
