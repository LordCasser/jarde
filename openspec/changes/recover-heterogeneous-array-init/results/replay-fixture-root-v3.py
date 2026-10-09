from pathlib import Path
import subprocess,hashlib,json,time,zipfile
REPO=Path('/Users/lordcasser/workspace/projects/jarde'); BASE=Path('/private/tmp/jarde-em18-baseline-20261009');FIX=REPO/'tests/fixtures/p3-heterogeneous-array-initializers-v3';OUT=Path(__file__).resolve().parent /'candidate-v1-fixture-v3';OUT.mkdir(exist_ok=True)
CLI=Path('/private/tmp/jarde-em18-candidate-v1-cli');h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();commands=[];cases=[]
def run(label,argv,cwd):
 a=OUT/'logs'/(label.replace('/','_')+'.stdout');b=a.with_suffix('.stderr');a.parent.mkdir(exist_ok=True)
 r=subprocess.run(list(map(str,argv)),cwd=cwd,stdout=subprocess.PIPE,stderr=subprocess.PIPE);a.write_bytes(r.stdout);b.write_bytes(r.stderr)
 row={'label':label,'argv':list(map(str,argv)),'cwd':str(cwd),'exit':r.returncode,'stdout':str(a.relative_to(OUT)),'stderr':str(b.relative_to(OUT)),'stdout_sha256':h(a),'stderr_sha256':h(b)};commands.append(row);return r,row
jobs=[]
b=json.loads((BASE/'manifest.json').read_text());f=json.loads((FIX/'build-manifest-v3-final.json').read_text())
for fam,row in f['families'].items():
 for leg,detail in row['legs'].items():
  jar=OUT/(fam+'-'+leg+'.jar')
  with zipfile.ZipFile(jar,'w')as z:
   for cf in sorted((FIX/fam/leg/'classes').glob('*.class')):z.write(cf,cf.name)
  jobs.append(('fixture',fam,leg,jar,{'exit':detail['original_runtime_exit'],'stdout_sha256':detail['original_stdout_sha256'],'stderr_sha256':detail['original_stderr_sha256']}))
for dataset,fam,leg,jar,orig in jobs:
 prefix=dataset+'/'+fam+'/'+leg;case=OUT/dataset/fam/leg;src=case/'sources';cls=case/'classes';empty=case/'empty';src.mkdir(parents=True,exist_ok=True);cls.mkdir(exist_ok=True);empty.mkdir(exist_ok=True)
 with zipfile.ZipFile(jar)as z:names=sorted(n[:-6].replace('/','.')for n in z.namelist()if n.endswith('.class'))
 reports=[]
 for name in names:
  r,rec=run(prefix+'/render-'+name,[CLI,'class-source','--input',jar,'--class',name,'--policy','plain-jar','--format','json','--evidence','all','--release','8'],REPO)
  obj=json.loads(r.stdout);sp=src/(name.replace('.','/')+'.java');sp.parent.mkdir(parents=True,exist_ok=True);sp.write_text(obj['text']);reports.append({'class':name,'source_sha256':h(sp),'bytecode_marker': '@bytecode' in obj['text'],'refused_body_marker':'jarde_refused_body' in obj['text'],'report_stdout':rec['stdout']})
 home=Path(b['legs'][leg]['home']);r,compile=run(prefix+'/compile',[home/'bin/javac','-source','8','-target','8','-g:none','-classpath',empty,'-sourcepath',empty,'-d',cls,*sorted(src.rglob('*.java'))],case)
 runtime=None
 if r.returncode==0:
  main='CT'if fam.startswith('ct-legacy')else'Main';_,runtime=run(prefix+'/run',[home/'bin/java','-Xverify:all','-cp',cls,main],case)
 matches=runtime is not None and runtime['exit']==orig['exit']==0 and all(runtime[s+'_sha256']==orig[s+'_sha256']for s in ['stdout','stderr'])
 quality=all(not x['bytecode_marker']and not x['refused_body_marker']for x in reports)
 cases.append({'dataset':dataset,'family':fam,'leg':leg,'input_jar':str(jar),'input_jar_sha256':h(jar),'original_run':orig,'reports':reports,'compile_exit':compile['exit'],'runtime':runtime,'matches_original_streams':matches,'all_sources_without_refusal':quality,'accepted':matches and quality});print(prefix,compile['exit'],matches,quality,flush=True)
 meta={'schema':'root-em18-candidate-v1-fixture-v3','runner_sha256':h(Path(__file__)),'cli_sha256':h(CLI),'cases':cases,'commands':commands};(OUT/'manifest.json').write_text(json.dumps(meta,indent=2)+'\n')
meta['files']=[{'path':str(p.relative_to(OUT)),'sha256':h(p),'bytes':p.stat().st_size}for p in sorted(OUT.rglob('*'))if p.is_file()and p.name!='manifest.json'];(OUT/'manifest.json').write_text(json.dumps(meta,indent=2)+'\n')
