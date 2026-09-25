#!/bin/sh
set -eu

fixture_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/jarde-anonymous-top-level-run.XXXXXX")
trap 'python3 -c '\''import shutil,sys; shutil.rmtree(sys.argv[1])'\'' "$temp_dir"' EXIT HUP INT TERM
mkdir -p "$temp_dir/classes"
javac --release 8 -g -d "$temp_dir/classes" "$fixture_dir/AnonymousTopLevel.java"
java -Xverify:all -cp "$temp_dir/classes" AnonymousTopLevel
