from pathlib import Path
import json,hashlib,re,sys
r=Path(__file__).resolve().parent;repo=r.parents[3];checks=0;issues=[]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def check(v,m):
 global checks
 checks+=1
 if not v:issues.append(m)
d=json.loads((r/'gates-v2/index.json').read_text())
expected=['focused-varargs','workspace-seed1','workspace-seed2','cargo-clean-mid-gates','p5-after-effect-fix','msrv-188','fmt','ci-clippy','ignored-p3','ignored-constructor','strict-openspec','diff-check']
check([x['name'] for x in d['stages']]==expected,'all stages actually ran')
check(d['ci_yml_sha256']==sha(repo/'.github/workflows/ci.yml'),'CI flags source')
check(d['runner_sha256']==sha(r/'composition-run-local-gates-root-v3.py'),'runner identity')
counts={}
for stage in d['stages']:
 check(stage['exit']==0,stage['name']+' exit')
 for s in ['stdout','stderr']:check(sha(r/'gates-v2'/stage[s])==stage[s+'_sha256'],stage['name']+' '+s)
 check(stage['env']['CARGO_BUILD_JOBS']=='1' and stage['env']['CARGO_INCREMENTAL']=='0' and stage['env']['RUST_TEST_THREADS']=='1',stage['name']+' serial env')
 if stage['name'].startswith('workspace-seed'):
  rows=re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;', (r/'gates-v2'/stage['stdout']).read_text())
  totals=[sum(int(x[i]) for x in rows) for i in range(3)]
  counts[stage['name']]={'targets':len(rows),'passed':totals[0],'failed':totals[1],'ignored':totals[2]}
  check(len(rows)==351 and totals==[3328,0,93],stage['name']+' exact totals')
cli=json.loads((r/'candidate-cli-v2.json').read_text())
check(sha(Path(cli['cli_path']))==cli['cli_sha256'],'frozen CLI2')
for file,h in cli['candidate_sources'].items():check(sha(repo/file)==h,'final product source '+file)
for label in ['corpus-gates-v1','focused-v7']:
 meta=json.loads((r/label/'index.json').read_text())
 for stage in meta['stages']:
  check(stage['exit']==0,label+' '+stage['name']+' exit')
  for s in ['stdout','stderr']:check(sha(r/label/stage[s])==stage[s+'_sha256'],label+' '+stage['name']+' '+s)
result={'checks':checks,'issues':issues,'workspace':counts,'gate_index_sha256':sha(r/'gates-v2/index.json'),'cli_sha256':cli['cli_sha256'],'historical_failed_seed_preserved':(r/'gates-v1/workspace-seed1-summary.json').exists()}
(r/'local-gates-root-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));sys.exit(bool(issues))
