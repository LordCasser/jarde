import hashlib,json,sys
from pathlib import Path
r=Path(__file__).resolve().parent
out=r/'current-finally-v1'
d=json.loads((out/'result.json').read_text());inventory=json.loads((out/'inventory.json').read_text());checks=0

def check(ok,label):
 global checks
 checks+=1
 assert ok,label

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
closed={row['path'] for row in inventory['files']}
check(len(closed)==len(inventory['files']),'unique closed files')
check(closed=={p.relative_to(out).as_posix() for p in out.rglob('*') if p.is_file() and p.name!='inventory.json'},'complete file set')
for row in inventory['files']:
 p=out/row['path'];check(not p.is_symlink() and p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],'file '+row['path'])
check(d['completed'] and d['current_sources_unchanged_during_probe'],'completed unchanged source snapshot')
cli=d['candidate_cli'];check(sha(Path(cli['path']))==cli['sha256'],'frozen CLI')
commands={c['label']:c for c in d['commands']}
check(len(commands)==19,'19 actual commands')
for c in commands.values():
 check(c['exit']==0,'command '+c['label'])
 for stream in ['stdout','stderr']:
  row=c[stream];p=out/row['path'];check(sha(p)==row['sha256'] and p.stat().st_size==row['bytes'],'stream '+c['label']+' '+stream)
replay=d['replay'];check(replay['render-all']['source_sha256']==replay['render-essential']['source_sha256'],'evidence-independent class text')
bcis={0,3,6,7,10,11,14,15,16,19,20,23,24,25,26,29,30}
for mode in ['all','essential']:
 doc=json.loads((out/'candidate'/mode/'document.json').read_text());method=next(m for m in doc['methods'] if bytes(m['item']['name']['raw'])==b'run' and bytes(m['item']['descriptor']['raw'])==b'()I')
 body=method['outcome']['report'];check(body['quality']=='structured' and body['representation']=='java','complete run '+mode)
 check(body['text'].count('cleanup();')==1 and 'finally {' in body['text'],'one cleanup '+mode)
 observed=set()
 for segment in body['source_map']['segments']:
  origin=segment['origin'];observed.add(origin['primary']['bci']);observed.update(x['bci'] for x in origin.get('derived',[]))
 check(bcis<=observed,'protected/cleanup/return source '+mode)
 for leg in ['javac8','javac23']:
  row=replay[f'candidate-{mode}-{leg}'];check(row['compile_exit']==0 and row['run_exit']==0 and all(row[k] for k in ['matches_original_exit','matches_original_stdout','matches_original_stderr']),f'four-path match {mode}/{leg}')
  orig=commands[f'original-{leg}-run'];cand=commands[f'candidate-{mode}-{leg}-run']
  for stream in ['stdout','stderr']:check((out/orig[stream]['path']).read_bytes()==(out/cand[stream]['path']).read_bytes(),f'actual raw bytes {mode}/{leg}/{stream}')
  compiler=commands[f'candidate-{mode}-{leg}-compile']['argv'];empty=Path(compiler[compiler.index('-classpath')+1]);check(empty==Path(compiler[compiler.index('-sourcepath')+1]) and not list(empty.iterdir()),'empty CP/SP')
  java=cand['argv'];check('-Xverify:all' in java and java[java.index('-cp')+1]==compiler[compiler.index('-d')+1],'fresh only runtime')
for leg in ['javac8','javac23']:
 orig=(out/commands[f'original-{leg}-run']['stdout']['path']).read_bytes();jadx=(out/commands[f'jadx-historical-{leg}-run']['stdout']['path']).read_bytes()
 check(len(orig.splitlines())==4 and b'cleanup=true:trace=29\n' in orig,'original four paths')
 check(b'cleanup=true:trace=299\n' in jadx and jadx!=orig,'historical JADX duplicate cleanup')
 original_class=Path(replay[f'original-{leg}']['class_copied_without_recompile']['path']);check(sha(original_class)=='924437916dc278eefe3b83cdcf3bad14bfb3cf8b9e44c027786a89f0712247b6','exact frozen original class')
wide=json.loads((out/'widened/document.json').read_text());m=next(x for x in wide['methods'] if bytes(x['item']['name']['raw'])==b'run');check('@bytecode' in m['outcome']['report']['text'] and 'finally {' not in m['outcome']['report']['text'],'widened run refused')
check(replay['widened-report-only']['executed'] is False,'mutant not executed')
result={'verification':'current_finally_complete_class_semantics_passed','checks':checks,'files':len(closed),'commands':len(commands),'cli_sha256':cli['sha256'],'probe_result_sha256':sha(out/'result.json'),'inventory_sha256':sha(out/'inventory.json'),'candidate_compiler_evidence_legs':4,'paths_per_leg':4,'historical_jadx_extraction_fresh':False,'historical_jadx_runtime_fresh':True,'remaining_boundary':'structured-subregion budget/cancellation/rollback coverage not implied by semantic positive','errors':[]}
p=r/'current-finally-root-verification-v1.json';assert not p.exists();p.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
