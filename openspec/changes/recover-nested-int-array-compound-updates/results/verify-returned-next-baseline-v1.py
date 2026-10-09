#!/usr/bin/env python3
"""Independent raw-byte/argv/full-source check for the next returned-update baseline."""
import hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;OUT=HERE/'returned-next-baseline-v1';ROOT=HERE.parents[3]
checks=0
def check(ok,label):
 global checks
 checks+=1
 assert ok,label
def sha(b):return hashlib.sha256(b).hexdigest()
m=json.loads((OUT/'manifest.json').read_bytes())
check(m['scope']=='next-candidate analysis only; no product/spec implementation','scope')
check(m['runner_sha256']==sha((HERE/'probe-returned-updates-baseline-v1.py').read_bytes()),'actual runner identity')
files={row['path']:row for row in m['files']}
check(set(files)=={p.relative_to(OUT).as_posix() for p in OUT.rglob('*') if p.is_file() and p.name!='manifest.json'},'full closed output inventory')
for name,row in files.items():
 b=(OUT/name).read_bytes();check(len(b)==row['bytes'] and sha(b)==row['sha256'],name)
expected=(OUT/m['expected_stdout']['path']).read_bytes()
check(len(expected.splitlines())==10 and b'ok:replace-row:17:100:17\n' in expected,'new value and old row oracle')
check(len(m['cases'])==8 and len(m['commands'])==23,'exact complete case and actual subprocess count')
for tools in m['jdk_tools']:
 for name,row in tools.items():check(sha(Path(row['path']).read_bytes())==row['sha256'],'real frozen '+name)
for c in m['cases']:
 label=c['label'];comp=c['compile'];argv=comp['argv'];cp=Path(argv[argv.index('-classpath')+1]);sp=Path(argv[argv.index('-sourcepath')+1]);dest=Path(argv[argv.index('-d')+1])
 check(cp.is_dir() and sp.is_dir() and not any(cp.iterdir()) and not any(sp.iterdir()),label+' empty CP/SP')
 check(argv[1:argv.index('-classpath')]==['-source','8','-target','8','-g:none','-Xlint:-options'],label+' actual compiler flags')
 sourcepaths=[str(OUT/r['path']) for r in c['sources']]
 check(argv[argv.index('-d')+2:]==sourcepaths and len(sourcepaths)==2,label+' all full target and fixed Runner source')
 harness=next(OUT/r['path'] for r in c['sources'] if Path(r['path']).name=='Runner.java');target=next(OUT/r['path'] for r in c['sources'] if Path(r['path']).name=='ReturnedIntArrayUpdates.java')
 runner=(OUT/'Runner.java').read_bytes();prefix=('package '+c['runner_package_prefix']+';\n').encode() if c.get('runner_package_prefix') else b''
 check(harness.read_bytes()==prefix+runner,label+' exact external oracle body')
 if c['kind']=='jarde':
  check(comp['exit']==1 and c['runtime'] is None and not c['success'],label+' preserve whole-class refusal/compile failure')
  raw=json.loads((OUT/c['decompile']['stdout']['path']).read_bytes());check(len(raw['methods'])==11,label+' all physical members')
  check(raw['text'].encode()==target.read_bytes(),label+' complete renderer text')
 else:
  run=c['runtime'];check(comp['exit']==0 and run is not None and run['exit']==0,label+' actual compile/run success')
  check((OUT/run['stdout']['path']).read_bytes()==expected and (OUT/run['stderr']['path']).read_bytes()==b'',label+' actual raw streams')
  check(run['argv'][0]==str(Path(argv[0]).parent/'java') and '-Xverify:all' in run['argv'] and run['argv'][run['argv'].index('-cp')+1]==str(dest),label+' own JDK and fresh verified classes only')
  check(sorted(p.name for p in dest.rglob('*.class'))==['ReturnedIntArrayUpdates.class','Runner.class'],label+' complete generated class closure')
  check(c['success'] is True,label+' summary agrees with raw success')
check(sum(c['success'] for c in m['cases'] if c['kind']=='original')==2,'original 2/2')
check(sum(c['success'] for c in m['cases'] if c['kind']=='jadx')==4,'fresh JADX 4/4')
check(sum(c['success'] for c in m['cases'] if c['kind']=='jarde')==0,'fresh current Jarde 0/2; no candidate execution')
p=HERE/'returned-next-root-verification-v1.json';check(not p.exists(),'no overwrite')
r={'status':'baseline_verified','checks':checks,'errors':[],'manifest_sha256':sha((OUT/'manifest.json').read_bytes()),'verifier_sha256':sha(Path(__file__).read_bytes()),'original':'2/2','fresh_jadx':'4/4','current_jarde':'0/2 complete-source compile failures; no runtime','scope':'next-candidate baseline only; no implementation acceptance'}
p.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
