#!/usr/bin/env python3
"""Fresh next-candidate analysis only; no product edits or implementation acceptance."""
import hashlib,json,os,re,shutil,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
RESULTS=Path(__file__).resolve().parent
OUT=RESULTS/'returned-next-baseline-v1'
assert not OUT.exists();OUT.mkdir()
def sha(b):return hashlib.sha256(b).hexdigest()
def row(p):return {'path':str(p.relative_to(OUT)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())}
commands=[]
def run(label,argv,home):
 env={k:v for k,v in os.environ.items() if k not in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')}
 if home:env['JAVA_HOME']=str(home);env['PATH']=str(home/'bin')+os.pathsep+env.get('PATH','')
 p=subprocess.run([str(x) for x in argv],cwd=ROOT,env=env,capture_output=True)
 streams={}
 for name,b in [('stdout',p.stdout),('stderr',p.stderr)]:
  f=OUT/'streams'/f'{label}.{name}';f.parent.mkdir(exist_ok=True);f.write_bytes(b);streams[name]=row(f)
 c={'label':label,'argv':[str(x) for x in argv],'java_home':str(home) if home else None,'exit':p.returncode,**streams};commands.append(c);return p,c
oldroot=ROOT/'tests/fixtures/nested-int-array-compound-updates'
source=(oldroot/'NestedIntUpdates.java').read_text().replace('NestedIntUpdates','ReturnedIntArrayUpdates')
for name in ['plain2','plain3','scalar','traced']:
 needle='public static void '+name;assert source.count(needle)==1;source=source.replace(needle,'public static int '+name)
for expression in ['a[i][j] += x;','a[i][j][k] += x;','a[i] += x;','a[row(r)][index(i)] += rhs(x);']:
 assert source.count(expression)==1;source=source.replace(expression,'return '+expression)
needle='a[0][0] += swap(a);\n        return 7;';assert source.count(needle)==1;source=source.replace(needle,'return a[0][0] += swap(a);')
runner=(oldroot/'Runner.java').read_text().replace('NestedIntUpdates','ReturnedIntArrayUpdates')
for var,call,value in [('result2','plain2(two, 1, 1, 5)',35),('result3','plain3(three, 1, 0, 1, 8)',14),('result1','scalar(one, 0, 5)',14),('resultT','traced(traced, 0, 0, 5)',15)]:
 needle='        ReturnedIntArrayUpdates.'+call+';';assert runner.count(needle)==1
 runner=runner.replace(needle,f'        int {var} = ReturnedIntArrayUpdates.{call};\n        if ({var} != {value}) throw new AssertionError("returned-value:" + {var});')
assert runner.count('result == 7')==1;runner=runner.replace('result == 7','result == 17')
sourcepath=OUT/'ReturnedIntArrayUpdates.java';sourcepath.write_text(source)
runnerpath=OUT/'Runner.java';runnerpath.write_text(runner)
expected=(oldroot/'javac8.original.stdout').read_bytes().replace(b'ok:replace-row:17:100:7\n',b'ok:replace-row:17:100:17\n')
(OUT/'expected.stdout').write_bytes(expected)
control=json.loads((RESULTS/'controls-v1/manifest.json').read_bytes())
meta=json.loads((RESULTS/'candidate-cli-v1.json').read_bytes());cli=Path(meta['cli_path']);assert sha(cli.read_bytes())==meta['cli_sha256']
for name,h in meta['candidate_sources'].items():assert sha((ROOT/name).read_bytes())==h
jdk23=Path(control['legs'][1]['jdk_tools']['java']['path']).parent.parent
jadx=Path('/opt/homebrew/bin/jadx');p,version=run('jadx-version',[jadx,'--version'],jdk23);assert p.returncode==0 and p.stdout==b'1.5.6\n'
cases=[];inputs=[]
def compile_run(label,sources,home,main):
 case=OUT/'cases'/label;case.mkdir(parents=True,exist_ok=True);empty=case/'empty';empty.mkdir();classes=case/'classes';classes.mkdir()
 p,compile=run(label+'-compile',[home/'bin/javac','-source','8','-target','8','-g:none','-Xlint:-options','-classpath',empty,'-sourcepath',empty,'-d',classes,*sources],home)
 runtime=None
 if p.returncode==0:p,runtime=run(label+'-run',[home/'bin/java','-Xverify:all','-cp',classes,main],home)
 success=bool(runtime and p.returncode==0 and p.stdout==expected and p.stderr==b'')
 return {'label':label,'compile':compile,'runtime':runtime,'sources':[row(p) for p in sources],'classes':[row(p) for p in sorted(classes.rglob('*.class'))],'success':success},classes
for leg in control['legs']:
 label=leg['leg'];tools=leg['jdk_tools']
 for t in tools.values():assert sha(Path(t['path']).read_bytes())==t['sha256']
 home=Path(tools['java']['path']).parent.parent
 original,classes=compile_run(label+'-original',[sourcepath,runnerpath],home,'Runner');assert original['success'];original['kind']='original';cases.append(original)
 inp=classes/'ReturnedIntArrayUpdates.class';inputs.append(row(inp))
 _,javap=run(label+'-javap',[home/'bin/javap','-p','-c','-s','-v',inp],home);assert javap['exit']==0
 for profile in ['default','none']:
  name=label+'-jadx-'+profile;case=OUT/'cases'/name;case.mkdir();dest=case/'jadx'
  argv=[jadx,'--no-res','--config','none','--threads-count','1']
  if profile=='none':argv+=['--rename-flags','none']
  _,decompile=run(name+'-decompile',[*argv,'-d',dest,inp],jdk23);assert decompile['exit']==0
  sources=sorted(dest.rglob('*.java'));assert len(sources)==1 and sources[0].name=='ReturnedIntArrayUpdates.java'
  text=sources[0].read_text();match=re.search(r'^package ([\w.]+);',text,re.M);package=match.group(1) if match else None
  copied=case/'Runner.java';copied.write_text(('package '+package+';\n' if package else '')+runner)
  candidate,_=compile_run(name,[*sources,copied],home,(package+'.' if package else '')+'Runner')
  candidate.update(kind='jadx',profile=profile,decompile=decompile,runner_package_prefix=package);cases.append(candidate)
 name=label+'-jarde';case=OUT/'cases'/name;case.mkdir()
 p,decompile=run(name+'-render',[cli,'class-source','--input',inp,'--class','ReturnedIntArrayUpdates','--policy','single-class','--release','8','--format','json','--evidence','all'],None);assert p.returncode==0
 document=json.loads(p.stdout);assert len(document['methods'])==11
 generated=case/'ReturnedIntArrayUpdates.java';generated.write_text(document['text']);copied=case/'Runner.java';copied.write_text(runner)
 candidate,_=compile_run(name,[generated,copied],home,'Runner');candidate.update(kind='jarde',decompile=decompile,physical_member_count=11);cases.append(candidate)
m={'schema':'returned-array-update-next-baseline-v1','scope':'next-candidate analysis only; no product/spec implementation','runner_sha256':sha(Path(__file__).read_bytes()),'candidate_cli_sha256':meta['cli_sha256'],'expected_stdout':row(OUT/'expected.stdout'),'source':row(sourcepath),'runner':row(runnerpath),'jadx_version':version,'jdk_tools':[x['jdk_tools'] for x in control['legs']],'inputs':inputs,'commands':commands,'cases':cases,'files':[row(p) for p in sorted(OUT.rglob('*')) if p.is_file()]}
(OUT/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
print(json.dumps({'original':sum(c['success'] for c in cases if c['kind']=='original'),'jadx':sum(c['success'] for c in cases if c['kind']=='jadx'),'jarde':sum(c['success'] for c in cases if c['kind']=='jarde'),'commands':len(commands)}))
