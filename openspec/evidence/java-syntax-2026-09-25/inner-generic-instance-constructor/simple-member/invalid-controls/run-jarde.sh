#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    printf 'usage: sh run-jarde.sh /path/to/jarde-cli\n' >&2
    exit 2
fi
cli=$1
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
shasum -a 256 "$cli" > "$root/jarde-cli-sha256.txt"
"$cli" class-source --input "$root/full-target.jar" --class nested/UseInner \
    --format json --output "$root/jarde-full-UseInner.json"
"$cli" class-source --input "$root/missing-target.jar" --class nested/UseInner \
    --format json --output "$root/jarde-missing-UseInner.json"
"$cli" class-source --input "$root/full-target.jar" --class nested/EffectOrder \
    --format json --output "$root/jarde-full-EffectOrder.json"
