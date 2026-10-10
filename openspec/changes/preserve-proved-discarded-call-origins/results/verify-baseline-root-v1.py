from pathlib import Path
import json,hashlib,re,stat
from blake3 import blake3
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');HERE=Path(__file__).resolve().parent;OUT=HERE/'baseline-root-v1'
sha=lambda b:hashlib.sha256(b).hexdigest()
d=json.loads((OUT/'execution.json').read_bytes());assert d['status']=='observed-baseline-not-accepted' and len(d['commands'])==26
by={x['label']:x for x in d['commands']};assert len(by)==26
assert d['cli']=={'path':'/private/tmp/jarde-return-arm-latch-cli-v1','sha256':'9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f'}
assert sha(Path(d['cli']['path']).read_bytes())==d['cli']['sha256'] and stat.S_IMODE(Path(d['cli']['path']).stat().st_mode)==0o555
assert sha(Path(d['jadx']['path']).read_bytes())==d['jadx']['sha256']
for name,h in d['inputs'].items():assert sha((HERE/'original-inputs-root-v1'/name).read_bytes())==h
for row in d['commands']:
 assert row['exit_code']==0 and row['guard_stop'] is None and row['cwd']==str(ROOT)
 for rec in row['streams'].values():
  path=Path(rec['path']);b=(path if path.is_absolute() else ROOT/path).read_bytes();assert len(b)==rec['bytes'] and sha(b)==rec['sha256']
def raw(label,key='stdout'):
 p=Path(by[label]['streams'][key]['path']);return (p if p.is_absolute() else ROOT/p).read_bytes()
def method_bcis(data):
 result={};name=None;descriptor=None;current=None
 for line in data.decode().splitlines():
  if re.match(r'^  (public|private|protected) .*\);$',line):
   header=line.strip().split('(')[0].split()[-1];name='<init>' if '.' in header else header;current=None
  elif name and (m:=re.match(r'^    descriptor: (.*)$',line)):
   descriptor=m[1];current=result.setdefault((name,descriptor),set())
  elif current is not None and (m:=re.match(r'^\s+(\d+): [a-z][a-z0-9_]*(?:\s|$)',line)):
   current.add(int(m[1]))
 return result
expected={('discardStatic','(Z)V'):{4},('discardAppend','(Ljava/lang/String;)Ljava/lang/String;'):{13},('discardListAdd','(Ljava/lang/String;)Ljava/lang/String;'):{15}}
oracles=[];profiles=[]
for leg,lr in d['legs'].items():
 home=Path('/Library/Java/JavaVirtualMachines/'+('corretto-1.8.0_432' if leg=='javac8' else 'openjdk-23.0.1')+'/Contents/Home')
 for name,rec in lr['tools'].items():assert Path(rec['path'])==home/'bin'/name and sha(Path(rec['path']).read_bytes())==rec['sha256']
 for rel,h in lr['original_classes'].items():assert sha((OUT/leg/'original'/rel).read_bytes())==h
 oracle=(raw(leg+'-original-runtime'),raw(leg+'-original-runtime','stderr'));oracles.append(oracle)
 assert oracle== (b'static-normal=completed\nstatic-throw=give-failed\nappend=appended\nlist-add=listed\nreturn-consumed=given\nlocal-deferred=given\n',b'')
 physical=method_bcis(raw(leg+'-DiscardedCallSourceProbe-javap'));assert len(physical)==7
 default=json.loads((OUT/leg/'default/class.json').read_bytes());all_=json.loads((OUT/leg/'all/class.json').read_bytes())
 cb=(OUT/leg/'original/discardprobe/DiscardedCallSourceProbe.class').read_bytes();identity={'digest':blake3(cb).hexdigest(),'length':len(cb)}
 assert default['class']['class_bytes']==identity and all_['class']['class_bytes']==identity
 assert default['text']==all_['text'] and '@bytecode' not in default['text']
 for profile in ['default','all','jadx']:
  assert (raw(leg+'-'+profile+'-runtime'),raw(leg+'-'+profile+'-runtime','stderr'))==oracle
  cr=by[leg+'-'+profile+'-compile']['argv'];compiled=str(OUT/leg/profile/'classes');assert cr[cr.index('-classpath')+1]==compiled and cr[cr.index('-d')+1]==compiled
  rt=by[leg+'-'+profile+'-runtime']['argv'];assert rt==[str(home/'bin/java'),'-Xverify:all','-cp',compiled,'discardprobe.DiscardedCallSourceProbeRunner']
 for doc in [default,all_]:
  found={}
  for meth in doc['methods']:
   item=meth['item'];key=(item['name']['escaped'],item['descriptor']['escaped']);pid=item['identity'];report=meth['outcome']['report'];assert meth['outcome']['kind']=='recovered' and report['artifact']['binding']['method']==pid
   assert pid['owner']['class_bytes']==identity and bytes(pid['name']).decode()==key[0] and bytes(pid['descriptor']).decode()==key[1]
   covered=set();text=report['text'].encode()
   for segment in report['source_map']['segments']:
    assert 0<=segment['start']<=segment['end']<=len(text)
    for orig in [segment['origin']['primary'],*segment['origin']['derived']]:assert orig['method']==pid;covered.add(orig['bci'])
   missing=physical[key]-covered;assert missing==expected.get(key,set()),(leg,key,missing)
   found[key]=report['source_map']
  assert set(found)==set(physical)
  profiles.append({'leg':leg,'class_sha256':sha(cb),'missing_pop_bcis':{k[0]:sorted(v) for k,v in expected.items()},'maps':found})
 assert {m['item']['name']['escaped']:m['outcome']['report']['source_map'] for m in default['methods']}=={m['item']['name']['escaped']:m['outcome']['report']['source_map'] for m in all_['methods']}
assert oracles[0]==oracles[1]
result={'schema':'discarded-call-source-baseline-root-acceptance-v1','status':'accepted-baseline-observations-only','manifest_sha256':sha((OUT/'execution.json').read_bytes()),'commands':26,'complete_runtime_legs':8,'method_profiles':28,'expected_missing_pop_bcis':{k[0]:sorted(v) for k,v in expected.items()},'no_typed_product':True,'candidate_fix_accepted':False}
(OUT/'acceptance-root-v1.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
