import hashlib,json,re
from pathlib import Path
HERE=Path(__file__).resolve().parent
OUT=HERE/'baseline-root-v1'
m=json.loads((OUT/'manifest.json').read_bytes()); checks=0

def check(ok,label):
 global checks
 checks+=1
 assert ok,label

def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def path(row): return OUT/row['path']
def verify_file(row):
 p=path(row);check(p.is_file() and not p.is_symlink() and p.stat().st_size==row['bytes'] and digest(p)==row['sha256'],'file '+row['path'])

inventory=json.loads((OUT/'file-inventory.json').read_bytes());names={x['path'] for x in inventory}
check(len(names)==len(inventory),'unique inventory')
check(names=={p.relative_to(OUT).as_posix() for p in OUT.rglob('*') if p.is_file() and p.name!='file-inventory.json'},'closed inventory')
for row in inventory: verify_file(row)
check(m['status']=='completed' and all(x['ok'] for x in m['preflight']),'actual preflight and completion')
check(m['case_counts']=={'original':2,'jadx':4,'jarde':2} and m['success_counts']==m['case_counts'],'eight complete legs')
check(m['original_cross_jdk_equal'],'original cross-JDK output equal')
check(len(m['commands'])==31,'31 actual commands')
for c in m['commands']:
 check(c['exit']==0,'exit '+c['label'])
 for stream in ('stdout','stderr'): verify_file(c[stream])
expected_methods={('<init>','()V',1),('storeLiteral','()V',1),('replace','(LArrayFieldStore;I)V',9),('element','(IBI)B',10),('readValues','()[B',1),('resetTrace','()V',9),('readTrace','()I',9)}
expected_fields={('values','[B',2),('trace','I',10)}
raw=lambda item,key:bytes(item[key]['raw']).decode()
original={c['jdk_leg']:c for c in m['cases'] if c['kind']=='original'}
for c in m['cases']:
 compiler=c['compile']['argv'];runtime=c['runtime']['argv'];dest=Path(compiler[compiler.index('-d')+1]);empty=Path(compiler[compiler.index('-classpath')+1])
 check(empty==Path(compiler[compiler.index('-sourcepath')+1]) and empty.is_dir() and not list(empty.iterdir()),c['label']+' empty CP/SP')
 check(compiler[1:compiler.index('-classpath')]==['-source','8','-target','8','-g:none','-Xlint:-options'],c['label']+' real compiler flags')
 check(len(c['source_files'])==2 and len(c['classes'])==2,c['label']+' full target plus fixed Runner')
 check(compiler[compiler.index('-d')+2:]==[str(path(x)) for x in c['source_files']],c['label']+' complete exact source paths')
 check('-Xverify:all' in runtime and Path(runtime[runtime.index('-cp')+1])==dest,c['label']+' fresh verified runtime')
 check(Path(runtime[0]).parent==Path(compiler[0]).parent,c['label']+' own real JDK')
 oracle=original[c['jdk_leg']]['runtime'];check(c['runtime']['exit']==oracle['exit'],c['label']+' runtime exit')
 for stream in ('stdout','stderr'):check(path(c['runtime'][stream]).read_bytes()==path(oracle[stream]).read_bytes(),c['label']+' raw '+stream)
 if c['kind']=='jarde':
  d=json.loads(path(c['document']).read_bytes())
  check({(raw(x['item'],'name'),raw(x['item'],'descriptor'),x['item']['access_flags']) for x in d['methods']}==expected_methods,c['label']+' physical methods')
  check({(raw(x['item'],'name'),raw(x['item'],'descriptor'),x['item']['access_flags']) for x in d['fields']}==expected_fields,c['label']+' physical fields')
  check('@bytecode' not in d['text'] and 'jarde_refused_body' not in d['text'],c['label']+' no refusal')
  for member in d['methods']:
   body=member['outcome']['report'];check(body['quality']=='structured' and body['representation']=='java',c['label']+' structured '+raw(member['item'],'name'))
   name=raw(member['item'],'name')
   if name in ('storeLiteral','replace'):
    origins=set()
    for segment in body['source_map']['segments']:
     origin=segment['origin'];origins.add(origin['primary']['bci']);origins.update(x['bci'] for x in origin.get('derived',[]))
    required=({0,1,2,4,5,6,8,9,10,11,13,14,15,16,18,19,22} if name=='storeLiteral' else {0,1,2,4,5,6,7,9,10,13,14,15,16,17,19,20,23,24,25,26,27,29,30,33,34,37})
    check(required<=origins,c['label']+' exact fresh allocation/element/call/store/return BCI '+name)
stdout=path(original['javac8']['runtime']['stdout']).read_bytes()
check(len(stdout.splitlines())==4 and b':fresh=true:trace=123\n' in stdout,'fresh identity and effect order')
check(b':same=true:values=[10, 20, 30]:trace=12\n' in stdout,'failed RHS preserves exact prior array')
check(b'null=NullPointerException:trace=123\n' in stdout and b'null-failure=IllegalStateException:element-2:trace=12\n' in stdout,'RHS before null store and exception priority')
result={'status':'accepted','checks':checks,'errors':[],'commands':31,'original_legs':2,'fresh_jadx_legs':4,'jarde_legs':2,'paths_per_leg':4,'manifest_sha256':digest(OUT/'manifest.json'),'inventory_sha256':digest(OUT/'file-inventory.json'),'cli_sha256':m['frozen_cli']['sha256'],'scope':'method fresh byte array initializer consumed by field store; complete class, physical seven methods/two fields and real array origins; no implementation gap demonstrated'}
p=HERE/'root-verification-v1.json';check(not p.exists(),'new verifier output');result['checks']=checks;p.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
