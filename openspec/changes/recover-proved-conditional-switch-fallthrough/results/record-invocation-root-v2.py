#!/usr/bin/env python3
"""Run one exact command and preserve its invocation, cwd, source pin and raw streams."""
from __future__ import annotations
import datetime, hashlib, json, subprocess, sys, time
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')

def sha(raw:bytes)->str:return hashlib.sha256(raw).hexdigest()
def write(path:Path,data:bytes):
    with path.open('xb') as f:f.write(data)
def main():
    if len(sys.argv)<4: raise SystemExit('usage: recorder OUT COMMAND [ARG ...]')
    out=Path(sys.argv[1]).resolve()
    if out.exists(): raise FileExistsError(f'refusing to overwrite invocation record: {out}')
    out.mkdir(parents=True)
    argv=sys.argv[2:]
    recorder=Path(__file__).resolve()
    row={'schema':'conditional-ci-invocation-record-root-v2','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
         'cwd':str(ROOT),'recorder':{'path':str(recorder),'sha256':sha(recorder.read_bytes())},
         'argv':argv,'argv_sha256':sha(json.dumps(argv,separators=(',',':')).encode()),'exit_code':None,'streams':{}}
    start=time.monotonic()
    try:
        with (out/'stdout.raw').open('xb') as stdout,(out/'stderr.raw').open('xb') as stderr:
            proc=subprocess.run(argv,cwd=ROOT,stdout=stdout,stderr=stderr,check=False)
        row['exit_code']=proc.returncode
    except OSError as exc:
        (out/'stdout.raw').touch(exist_ok=True);(out/'stderr.raw').write_text(f'{type(exc).__name__}: {exc}')
        row['exit_code']=None;row['spawn_error']=f'{type(exc).__name__}: {exc}'
    row['duration_seconds']=time.monotonic()-start
    for name in ('stdout','stderr'):
        raw=(out/(name+'.raw')).read_bytes()
        row['streams'][name]={'path':name+'.raw','bytes':len(raw),'sha256':sha(raw)}
    (out/'execution.json').write_text(json.dumps(row,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(row,ensure_ascii=False))
    return 0 if row['exit_code']==0 else 1
if __name__=='__main__':raise SystemExit(main())
