from pathlib import Path
import hashlib,json,os,subprocess,zipfile
R=Path(__file__).resolve().parent; REPO=R.parents[3]
OUT=R/'nested-controls-cli2-root-v1';OUT.mkdir();(OUT/'logs').mkdir()
base=R/'control-drafts-v1/run-001';original=json.loads((base/'manifest.json').read_text());cli=Path('/private/tmp/jarde-em18-composition-cli-v2');jadx=Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
d={'cli_sha256':sha(cli),'original_manifest_sha256':sha(base/'manifest.json'),'commands':[],'cases':[]}
def save():
 d['files']=[{'path':str(p.relative_to(OUT)),'sha256':sha(p)} for p in sorted(OUT.rglob('*')) if p.is_file() and p.name!='manifest.json'];(OUT/'manifest.json').write_text(json.dumps(d,indent=2)+'\n')
def run(label,argv,cwd,env=None):
 argv=list(map(str,argv));p=subprocess.run(argv,cwd=cwd,env=env,capture_output=True);v={'label':label,'argv':argv,'cwd':str(cwd),'exit':p.returncode}
 for s in ['stdout','stderr']:
  q=OUT/'logs'/(label+'.'+s);q.write_bytes(getattr(p,s));v[s]=str(q.relative_to(OUT));v[s+'_sha256']=sha(q)
 d['commands'].append(v);save();return p,v
for leg,info in original['jdk_legs'].items():
 home=Path(info['jdk_home']);inputs=base/leg/'nested/classes';cls=inputs/'NestedControls.class';check=info['families']['nested']['classes'][0];assert sha(cls)==check['sha256']
 p,oracle=run(leg+'-original-run',[home/'bin/java','-Xverify:all','-cp',inputs,'NestedControls'],REPO);assert p.returncode==0;assert p.stdout==b'mark:nested\nnested\n' and p.stderr==b''
 jar=OUT/(leg+'.jar')
 with zipfile.ZipFile(jar,'w') as z:z.write(cls,'NestedControls.class')
 for profile in ['jarde','jadx-none','jadx-default']:
  work=OUT/(leg+'-'+profile);work.mkdir();sources=work/'sources';classes=work/'classes';classes.mkdir();empty=work/'empty';empty.mkdir()
  if profile=='jarde':
   sources.mkdir();p,record=run(leg+'-'+profile+'-render',[cli,'class-source','--input',jar,'--class','NestedControls','--policy','plain-jar','--format','json','--evidence','all','--release','8'],REPO);assert p.returncode==0;obj=json.loads(p.stdout);(sources/'NestedControls.java').write_text(obj['text']);quality='@bytecode' not in obj['text'] and 'jarde_refused_body' not in obj['text'];main='NestedControls'
  else:
   env=dict(os.environ,JAVA_HOME=original['jdk_legs']['javac23']['jdk_home']);args=[jadx,'--no-res','-d',work/'jadx-output'];args+=['--rename-flags','none'] if profile=='jadx-none' else [];p,_=run(leg+'-'+profile+'-decompile',[*args,jar],REPO,env);sources=work/'jadx-output/sources';files=list(sources.rglob('NestedControls.java'));assert len(files)==1;txt=files[0].read_text();main='defpackage.NestedControls' if 'package defpackage;' in txt else 'NestedControls';quality=all('UnsupportedOperationException("Method not decompiled' not in f.read_text() for f in sources.rglob('*.java'))
  files=sorted(sources.rglob('*.java'));assert len(files)==1
  p,compile=run(leg+'-'+profile+'-compile',[home/'bin/javac',*info['families']['nested']['compiler_flags'],'-g:none','-classpath',empty,'-sourcepath',empty,'-d',classes,*files],work)
  runtime=None
  if p.returncode==0:_,runtime=run(leg+'-'+profile+'-run',[home/'bin/java','-Xverify:all','-cp',classes,main],work)
  accepted=bool(runtime and runtime['exit']==0 and quality and all(runtime[s+'_sha256']==oracle[s+'_sha256'] for s in ['stdout','stderr']))
  d['cases'].append({'leg':leg,'profile':profile,'compile_exit':compile['exit'],'quality':quality,'runtime':runtime,'source_set':[str(f.relative_to(OUT)) for f in files],'accepted':accepted});save();print(leg,profile,accepted,flush=True)
save()
