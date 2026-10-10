from pathlib import Path
import hashlib,json,re,subprocess
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-typed-workspace-batches-root-v4');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
d=json.loads((out/'execution.json').read_text());assert d['status']=='passed' and d['pins_unchanged'];assert d['seed']=='5350648285461741569';assert d['guard_limits']=={'free_bytes':5*1024**3,'target_bytes':1024**3}
md=json.loads((out/'metadata.json').read_text());assert sha(out/'metadata.json')==d['metadata_sha256'];commands=[];expected=[];parts=[]
for p in md['packages']:
 base=['cargo','test','-p',p['name'],'--all-features','--locked','--message-format','json'];unit=[t for t in p['targets'] if t['kind'][0] in ('lib','bin','example','bench')]
 if unit:
  flags=[flag for kind,flag in [('lib','--lib'),('bin','--bins'),('example','--examples'),('bench','--benches')] if any(t['kind'][0]==kind for t in unit)];commands.append(base+flags);parts.append([(p['id'],t['name'],t['kind'][0],t['src_path']) for t in unit]);expected.extend((p['name'],t['name'],t['kind'][0]) for t in unit)
 tests=[t for t in p['targets'] if t['kind'][0]=='test']
 for start in range(0,len(tests),4):
  batch=tests[start:start+4];commands.append(base+sum((['--test',t['name']] for t in batch),[]));parts.append([(p['id'],t['name'],'test',t['src_path']) for t in batch]);expected.extend((p['name'],t['name'],'test') for t in batch)
assert sorted(expected)==sorted(tuple(x) for x in d['target_inventory']);assert len(expected)==len(set(expected));assert len(commands)==len(d['commands']);totals=[0,0,0];summaries=0;streams=0
for i,(row,argv,wanted) in enumerate(zip(d['commands'],commands,parts)):
 assert row['index']==i and row['argv']==argv and row['cwd']==str(root) and row['exit_code']==0 and row['guard_stop'] is None
 assert row['peak_target_bytes']<=1024**3 and row['free_bytes_after']>=5*1024**3
 raw={}
 for label,rec in row['streams'].items():
  p=Path(rec['path']);assert p.is_absolute() and p.stat().st_size==rec['bytes'] and sha(p)==rec['sha256'];raw[label]=p.read_text();streams+=1
 assert set(raw)=={'stdout','stderr'};counts=[list(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw['stdout'])];assert counts==row['observed_counts'] and len(counts)==len(wanted) and all(c[1]==0 for c in counts)
 assert 'test result: FAILED' not in raw['stdout'];artifacts=[]
 for line in raw['stdout'].splitlines():
  if not line.startswith('{'):continue
  a=json.loads(line)
  if a.get('reason')=='compiler-artifact' and a.get('profile',{}).get('test') and a.get('executable'):
   t=a['target'];artifacts.append((a['package_id'],t['name'],t['kind'][0],t['src_path']))
 assert sorted(artifacts)==sorted(wanted),(i,artifacts,wanted)
 stderr=re.sub(r'\x1b\[[0-9;]*m','',raw['stderr']);assert len(re.findall(r'(?m)^\s*Running ',stderr))==len(wanted),i
 summaries+=len(counts)
 for c in counts:
  for j,n in enumerate(c):totals[j]+=n
assert summaries==len(expected)==d['workspace_summary_count'];assert totals==d['workspace_totals']
product=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root).decode().strip();assert product=='43dffc9b806dc2b69102e50f52ee78a50850c6d0'
for path,h in d['source_pins'].items():assert sha(root/path)==h and hashlib.sha256(subprocess.check_output(['git','show',product+':'+path],cwd=root)).hexdigest()==h,path
for path,key in [('jarde-typed-workspace-batches-root-v3','carried_execution_sha256')]:assert sha(Path('/private/tmp')/path/'execution.json')==d[key]
assert not (root/'target').exists()
a={'schema':'typed-full-workspace-batched-acceptance-root-v1','status':'accepted-local-only','product_commit':product,'execution_sha256':sha(out/'execution.json'),'verifier_sha256':sha(__file__),'guarded_commands':len(commands),'metadata_targets_and_result_records':summaries,'raw_streams_verified':streams,'totals':totals,'source_pins_verified_against_git_and_live':len(d['source_pins']),'peak_target_bytes':max(r['peak_target_bytes'] for r in d['commands']),'scope':'One fixed seed, all metadata workspace targets and all features. Exact compile artifact and execution headers, unfiltered result summaries, carried raw SHA and source pins verified. CLI frozen unchanged; target cleaned. This does not replace own CI two seeds or ignored JDK oracles.'}
(out/'acceptance-root-v1.json').write_text(json.dumps(a,indent=2)+chr(10));print(json.dumps(a))
