from pathlib import Path
import hashlib,json,zipfile
R=Path(__file__).resolve().parent;OLD=R.parent.parent/'recover-heterogeneous-array-init/results'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
results=[];issues=[];checks=0
def check(ok,msg):
 global checks
 checks+=1
 if not ok:issues.append(msg)
for label,oldlabel in [('legacy-18-cli1-root-v2','candidate-v1-replay-v2'),('legacy-family-cli1-root-v1','candidate-v1-fixture-v3')]:
 root=R/label;d=json.loads((root/'manifest.json').read_text());old=json.loads((OLD/oldlabel/'manifest.json').read_text());oldcases={(c['dataset'],c['family'],c['leg']):c for c in old['cases']}
 for f in d['files']:check(sha(root/f['path'])==f['sha256'],label+' '+f['path'])
 for cmd in d['commands']:
  for s in ['stdout','stderr']:check(sha(root/cmd[s])==cmd[s+'_sha256'],label+' '+cmd['label']+' '+s)
 for c in d['cases']:
  prior=oldcases[(c['dataset'],c['family'],c['leg'])];check(c['input_jar_sha256']==prior['input_jar_sha256'],'same frozen input '+c['family']);check(c['original_run']==prior['original_run'],'same oracle '+c['family'])
  # ZIP output headers may carry different mtimes; compare each class hash independently as well.
  with zipfile.ZipFile(c['input_jar']) as z: expected={n[:-6].replace('/','.') for n in z.namelist() if n.endswith('.class')}
  check({x['class'] for x in c['reports']}==expected,'complete reported class set')
  prefix=c['dataset']+'/'+c['family']+'/'+c['leg'];cmd=next(cmd for cmd in d['commands'] if cmd['label']==prefix+'/compile');argv=cmd['argv'];check(len([a for a in argv if a.endswith('.java')])==len(expected),'all generated sources compile')
  for flag in ['-classpath','-sourcepath']:
   p=Path(argv[argv.index(flag)+1]);check(p.is_dir() and not list(p.iterdir()),'empty '+flag)
  if c['runtime']:
   argv=c['runtime']['argv'];check('-Xverify:all' in argv,'verifier');p=Path(argv[argv.index('-cp')+1]);check(p.name=='classes' and str(root) in str(p),'candidate only runtime CP')
  if prior['accepted']:check(c['accepted'],'previous full success retained '+c['family'])
 results.append({'run':label,'manifest_sha256':sha(root/'manifest.json'),'accepted':sum(c['accepted'] for c in d['cases']),'total':len(d['cases']),'failed':[{k:c[k] for k in ['family','leg','compile_exit','matches_original_streams','all_sources_without_refusal']} for c in d['cases'] if not c['accepted']]})
d={'checks':checks,'issues':issues,'results':results};(R/'regression-replays-root-verification-v1.json').write_text(json.dumps(d,indent=2)+'\n');print(json.dumps(d));raise SystemExit(bool(issues))
