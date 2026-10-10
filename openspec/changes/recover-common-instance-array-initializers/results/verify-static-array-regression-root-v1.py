from pathlib import Path
import hashlib,json,os
ROOT=Path(__file__).resolve().parents[4]
BASE=Path(__file__).resolve().parent
OUT=BASE/'static-array-regression-root-v3'
RESULT=BASE/'static-array-regression-root-acceptance-v1.json'
def sha(b): return hashlib.sha256(b).hexdigest()
def load(p): return json.loads(p.read_bytes())
def read(row):
 p=Path(row['path']);p=p if p.is_absolute() else OUT/p
 assert p.is_file() and not p.is_symlink()
 b=p.read_bytes();assert len(b)==row['bytes'] and sha(b)==row['sha256'];return b
assert not RESULT.exists()
m=load(OUT/'manifest.json'); inv=load(OUT/'file-inventory.json')
assert m['status']=='completed' and m['failures']==[] and len(m['cases'])==8 and len(m['commands'])==24
listed={r['path'] for r in inv};actual={p.relative_to(OUT).as_posix() for p in OUT.rglob('*') if p.is_file() and p.name!='file-inventory.json'}
assert len(listed)==len(inv) and listed==actual
for r in inv:read(r)
labels={r['label']:r for r in m['commands']};assert len(labels)==24
for r in m['commands']:
 assert r['exit']==0
 read(r['stdout']);read(r['stderr'])
assert all(m['profile_checks'].values())
for c in m['cases']:
 assert c['success'] and c['compile_success'] and c['runtime_success'] and c['exact_class_set'] and c['runtime_matches_original_raw']
 assert all(c['proof_checks'].values())
 d=load(OUT/c['document']['path']);assert d['text'].encode()==read(c['source_files'][0])
 assert c['render']==labels[c['render']['label']] and read(c['render']['stdout'])==read(c['document'])
 for command in [c['compile'],c['runtime']]:assert command==labels[command['label']]
 a=c['compile']['argv'];assert a[1:8]==['-source','8','-target','8','-g:none','-Xlint:-options','-classpath']
 assert a[8]==a[10] and a[9]=='-sourcepath' and list(Path(a[8]).iterdir())==[] and a[11]=='-d'
 assert {str(OUT/r['path']) for r in c['source_files']}==set(a[13:])
 assert c['runtime']['argv'][1:4]==['-Xverify:all','-cp',a[12]]
 classes=Path(a[12]); expected_prefix=c['runtime']['argv'][-1].rpartition('.')[0].replace('.','/')
 prefix=expected_prefix+'/' if expected_prefix else ''
 assert {p.relative_to(classes).as_posix() for p in classes.rglob('*.class')}=={prefix+c['source_files'][0]['path'].split('/')[-1].replace('.java','.class'),prefix+'Runner.class'}
 original=read(c['oracle_raw']['stdout']);assert read(c['runtime']['stdout'])==original
 assert read(c['runtime']['stderr'])==read(c['oracle_raw']['stderr']) and c['runtime']['exit']==c['oracle_raw']['exit']==0
runner_sha={'literal':'fd69838bf2a372d1bceb27b83fffc3b24b23a2ba1967bb51038e4473b1d4e163','ordered':'b6541a755e1e9b6c2c4b7d601b14fcc2a27f820650ec968fe0f2d150f79a28cd'}
for c in m['cases']:
 r=read(c['source_files'][1]);package=c['runner_package_adaptation']
 if package:
  prefix=('package '+package+';\n\n').encode();assert r.startswith(prefix);r=r[len(prefix):]
 assert sha(r)==runner_sha[c['family']]
r={'status':'accepted','manifest_sha256':sha((OUT/'manifest.json').read_bytes()),'inventory_sha256':sha((OUT/'file-inventory.json').read_bytes()),'closed_files':len(inv),'commands':24,'complete_source_runtime_legs':8,'physical_report_scope':'complete item/text/source_map equality checked by the root-reviewed collector; evidence requests and budget usage excluded','candidate_cli_sha256':m['candidate_cli']['sha256']}
RESULT.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
