#!/bin/sh
set -eu
fixture_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/jarde-anonymous-member-base.XXXXXX")
trap 'python3 -c '\''import shutil,sys; shutil.rmtree(sys.argv[1])'\'' "$temp_dir"' EXIT HUP INT TERM
javac --release 8 -g -d "$temp_dir" "$fixture_dir/AnonymousMemberBase.java"
java -Xverify:all -cp "$temp_dir" AnonymousMemberBase
