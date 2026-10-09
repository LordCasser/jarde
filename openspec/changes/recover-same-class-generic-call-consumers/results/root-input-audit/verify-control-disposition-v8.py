#!/usr/bin/env python3
"""Root check of frozen control facts against actual CLI8 transcripts and prior reviewed scope."""
import hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[4]
CLI_SHA='8770d823c70e7f61749cac836e468c0a991093822d1926822d4769adc1cf7339'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def key(x): return (x['case'],x['leg'],x['debug'])
facts_path=HERE/'candidate-v8-control-facts.json'
facts=json.loads(facts_path.read_text())
prior=json.loads((HERE/'candidate-v6-control-facts-v2.json').read_text())
review=json.loads((HERE/'candidate-v6-control-root-disposition.json').read_text())
old={key(x):x for x in prior['records']}
actual_root=HERE.parent/'candidate/candidate-v8/gc01-08-140'
manifest=json.loads((actual_root/'manifest.json').read_text())
actual={(x['name'],x['leg'],x['debug']):x for x in manifest['cases']}
errors=[];checks=[]
def need(ok,msg):
 if not ok: errors.append(msg)
need(sha(Path('/private/tmp/jarde-generic-calls-candidate-v8-cli'))==CLI_SHA,'CLI8 identity')
need(len(facts['records'])==44 and {key(x) for x in facts['records']}==set(old),'44 unique controls')
for row in facts['records']:
 k=key(row); before=old[k]; recorded=actual[k]; label='/'.join(k)
 for role,item in row['inputs'].items():
  need(sha(REPO/item['path'])==item['sha256']==before['inputs'][role]['sha256'],label+'/'+role)
 files=row['candidate_output_files']; olds=before['candidate_output_files']
 need([(x['class'],x['sha256']) for x in files]==[(x['class'],x['sha256']) for x in olds],label+'/reviewed-source-shape')
 for item in files: need(sha(REPO/item['path'])==item['sha256'],label+'/candidate-source')
 probe=row['probe']['candidate']; oldprobe=before['probe']['candidate']; observed=recorded['flavors']['candidate']
 area=actual_root/k[1]/k[2]/k[0]/'compile-sources/candidate'
 for stream in ('stdout','stderr'):
  text=(area/('candidate-probe.'+stream)).read_text()
  need(text==probe[stream] and sha(area/('candidate-probe.'+stream))==probe[stream+'_sha256']==observed['probe_'+stream+'_sha256'],label+'/probe-'+stream)
 need(probe['stdout']==oldprobe['stdout'] and probe['stdout_sha256']==oldprobe['stdout_sha256'],label+'/reviewed-transcript')
 need(probe['exit']==probe['compile_exit']==observed['probe_exit']==observed['compile_exit']==0,label+'/compile-verify')
 need(probe['reflection_matches_original'] is observed['reflection_matches_original'] and probe['reflection_matches_original'] is oldprobe['reflection_matches_original'],label+'/reflection')
 need(probe['reflection_sha256']==observed['reflection_sha256'],label+'/reflection-hash')
 need(probe['probe_failures']==observed['probe_failures']==[],label+'/behavior-failures')
 need(probe['behavior_lines']==observed['behavior_lines']==oldprobe['behavior_lines'],label+'/behavior')
 checks.append({'case':k[0],'leg':k[1],'debug':k[2],'input_hashes':{n:v['sha256'] for n,v in row['inputs'].items()},'candidate_files':files,'source_and_probe_equal_to_cli6':True,'reflection_matches_original':probe['reflection_matches_original']})
result={'scope':'Root independently verifies CLI8 controls against actual frozen inputs, complete emitted source, recompile/verify transcripts and the previously reviewed physical/source boundaries. Nine bounded refusals and two positive controls; no internal AST trace is inferred from markers.','candidate_cli_sha256':CLI_SHA,'facts_sha256':sha(facts_path),'facts_source':str(facts_path),'facts_records':len(facts['records']),'rows':review['rows'],'actual_checks':checks,'limitations':['Only v41/CLI8. Complete local gates and latest HEAD remote CI remain separate evidence.'],'errors':errors,'passed':not errors,'verifier_sha256':sha(__file__)}
out=HERE/'candidate-v8-control-root-disposition.json'
with out.open('x') as f: json.dump(result,f,indent=2,ensure_ascii=False); f.write('\n')
print(json.dumps({'passed':result['passed'],'checks':len(checks),'errors':errors},ensure_ascii=False))
raise SystemExit(0 if not errors else 1)
