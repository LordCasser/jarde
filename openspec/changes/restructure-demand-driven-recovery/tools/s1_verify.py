#!/usr/bin/env python3
"""对冻结 S0 输入重放 S1 二进制；不编译或执行真实目标包。"""
import gzip,hashlib,json,os,subprocess,sys
from pathlib import Path
import s0_baseline as b
out=Path(sys.argv[1]).resolve(); bins=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=True)
def clean(v):
    if isinstance(v,dict):return {k:clean(x) for k,x in v.items() if k!='elapsed_millis'}
    if isinstance(v,list):return [clean(x) for x in v]
    return v
checks=[]
for name in ['root-jarde','java-jarde_java']:
    binary=bins/name
    if not binary.exists():continue
    meta,text=b.run([binary,'--test-threads=1'],out/name,timeout=300)
    checks.append(dict(kind='unit',name=name,binary=b.pin(binary),exit_code=meta['exit_code'],summary=text.splitlines()[-2:]))
    b.write(out/'checks.json',checks)
    print(json.dumps(checks[-1]),flush=True)
cli=bins/'cli-jarde-cli'; comparisons=[]
for base in [b.CHANGE/'results/s0-baseline-v1',b.CHANGE/'results/s0-resumed-v2/expanded-matrix']:
    for row in json.loads((base/'matrix-results.json').read_text()):
        tag=row['id']+'-'+row['compiler']+'-'+row['debug'];source=base/'matrix'/tag
        old_meta=json.loads((source/'recover-jarde.meta.json').read_text());cmd=old_meta['command'];cmd[0]=str(cli)
        assert b.pin(Path(row['input']['path']))==row['input']
        meta,text=b.run(cmd,out/'matrix'/tag/'recover',timeout=120)
        baseline=json.loads(gzip.decompress((source/'recover-jarde.stdout.gz').read_bytes()));candidate=json.loads(text)
        comparisons.append(dict(id=tag,exit_code=meta['exit_code'],text_equal=baseline.get('text')==candidate.get('text'),report_equal_except_elapsed=clean(baseline)==clean(candidate)))
        b.compress(out/'matrix'/tag/'recover.stdout')
        b.write(out/'matrix-comparison.json',comparisons)
print(json.dumps(dict(unit_runs=len(checks),unit_failures=sum(x['exit_code']!=0 for x in checks),matrix_cases=len(comparisons),text_mismatches=sum(not x['text_equal'] for x in comparisons),report_mismatches=sum(not x['report_equal_except_elapsed'] for x in comparisons))),flush=True)
assert all(x['exit_code']==0 for x in checks)
assert len(comparisons)==136 and all(x['exit_code']==0 and x['report_equal_except_elapsed'] for x in comparisons)
