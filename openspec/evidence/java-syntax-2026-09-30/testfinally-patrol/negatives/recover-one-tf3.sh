#!/bin/sh
# Recovers `test()[B` from one class file with the jarde CLI and prints one line:
# quality + fallbacks. Used for the baseline/after refusal records.
set -eu
BIN=${1:?usage: recover-one.sh /path/to/jarde-cli <class-file>}
CLASS=$2
HERE=$(cd "$(dirname "$0")" && pwd)
DEF=$("$BIN" list-classes --input "$CLASS" --format json 2>/dev/null |
  python3 -c "import json,sys; print(json.dumps(json.load(sys.stdin)['items'][0]['definition']))")
METHOD=$("$BIN" list-members --input "$CLASS" --definition "$DEF" --format json 2>/dev/null |
  python3 -c "
import json,sys
doc = json.load(sys.stdin)
for m in doc['items']:
    ident = m.get('identity', {})
    if ident.get('descriptor') == [40, 41, 91, 66] and ident.get('name') == [116, 101, 115, 116]:
        print(json.dumps(ident))
        break
")
if [ -z "$METHOD" ]; then
  echo "no test()[B member in $CLASS"
  exit 1
fi
"$BIN" recover --input "$CLASS" --policy single-class --format json --evidence all \
  --method "$METHOD" 2>/dev/null |
  python3 -c "
import json, sys
doc = json.load(sys.stdin)
rec = doc['recovered']['recovery']
text = rec['text']
finally_count = text.count('finally {')
print(f\"quality={rec['quality']} finallys={finally_count} fallbacks={rec.get('fallbacks', [])}\")
"
