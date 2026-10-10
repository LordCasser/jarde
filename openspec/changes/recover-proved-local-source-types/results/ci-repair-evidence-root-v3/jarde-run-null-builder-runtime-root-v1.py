from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys,zipfile
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-null-builder-runtime-root-v1');out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
cli=Path('/private/tmp/jarde-proved-local-source-types-cli-v1');assert sha(cli)=='e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403'
fixture=root/'tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.class';assert sha(fixture)=='93acdfe27b312e2c98666a00dbfcb5f61299b03f62ac0dbd916b385232f260b3'
manifest=root/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json';assert sha(manifest)=='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
jdk=json.loads(manifest.read_text())['legs'];tools={leg['leg']:{k:Path(v['path']).resolve() for k,v in leg['jdk_tools'].items()} for leg in jdk}
for leg in jdk:
 for k,v in leg['jdk_tools'].items():assert sha(tools[leg['leg']][k])==v['sha256']
runner='public class Runner { public static void main(String[] args) { System.out.println("true="+NullThenBuilder.run(true)); System.out.println("false="+NullThenBuilder.run(false)); } }\n'
record={'schema':'meet-char-int-full-class-comparison-root-v1','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'fixture_sha256':sha(fixture),'cli_sha256':sha(cli),'original_source_available':False,'runner_sha256':hashlib.sha256(runner.encode()).hexdigest(),'commands':[],'legs':[],'status':'running'}
save=lambda:(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
env=os.environ.copy()
for k in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(k,None)
env.update(LC_ALL='C',TZ='UTC',JAVA_HOME=str(tools['javac23']['java'].parent.parent))
def run(argv):
 index=len(record['commands']);start=datetime.datetime.now(datetime.timezone.utc);p=subprocess.run(list(map(str,argv)),cwd=root,env=env,capture_output=True,timeout=60)
 row={'argv':list(map(str,argv)),'exit_code':p.returncode,'utc':start.isoformat(),'duration_seconds':(datetime.datetime.now(datetime.timezone.utc)-start).total_seconds()}
 for key,data in [('stdout',p.stdout),('stderr',p.stderr)]:
  path=out/f'{index}.{key}.raw';path.write_bytes(data);row[key]={'path':str(path),'bytes':len(data),'sha256':sha(path)}
 record['commands'].append(row);save();assert p.returncode==0,(argv,p.stderr.decode(errors='replace'));return p,row
save()
try:
 reports={}
 for profile in ('default','all'):
  argv=[cli,'class-source','--input',fixture,'--class','NullThenBuilder','--policy','single-class','--release','8','--format','json']
  if profile=='all':argv+=['--evidence','all']
  p,row=run(argv);report=json.loads(p.stdout);assert len(report['methods'])==3
  text=report['text'];assert 'java.lang.StringBuilder local2;' in text and 'local2 = null;' in text and 'local2 = new java.lang.StringBuilder("second");' in text
  reports[profile]=text
 assert reports['default']==reports['all']
 jar=out/'input.jar'
 with zipfile.ZipFile(jar,'w') as z:z.write(fixture,'NullThenBuilder.class')
 jadx=Path('/opt/homebrew/bin/jadx').resolve();assert sha(jadx)=='64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
 run([jadx,'--no-res','--config','none','--threads-count','1','-d',out/'jadx-output',jar]);jadx_source=out/'jadx-output/sources/defpackage/NullThenBuilder.java';assert jadx_source.exists()
 for leg,tool in tools.items():
  oracle=None
  for profile in ('original','default','all','jadx'):
   work=out/leg/profile;work.mkdir(parents=True);classes=work/'classes';classes.mkdir();empty=work/'empty';empty.mkdir()
   source=work/'NullThenBuilder.java';
   if profile=='original':shutil.copyfile(fixture,classes/'NullThenBuilder.class')
   else:source.write_bytes(jadx_source.read_bytes() if profile=='jadx' else reports[profile].encode())
   runnerpath=work/'Runner.java';runnerpath.write_text(('package defpackage;\n' if profile=='jadx' else '')+runner)
   flags=['-source','8','-target','8'] if leg=='javac8' else ['--release','8']
   run([tool['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8',*flags,'-g:none','-proc:none','-classpath',classes if profile=='original' else empty,'-sourcepath',empty,'-d',classes,*([runnerpath] if profile=='original' else [source,runnerpath])])
   p,row=run([tool['java'],'-Xverify:all','-cp',classes,'defpackage.Runner' if profile=='jadx' else 'Runner']);triple=(p.returncode,p.stdout,p.stderr)
   assert p.stdout==b'true=7\nfalse=6\n' and p.stderr==b''
   if oracle is None:oracle=triple
   else:assert triple==oracle
   record['legs'].append({'jdk':leg,'profile':profile,'source_sha256':None if profile=='original' else sha(source),'runner_sha256':sha(runnerpath),'runtime':row})
 record['status']='passed';record['full_class_methods_retained']=3
except BaseException as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:assert sha(cli)==record['cli_sha256'] and sha(fixture)==record['fixture_sha256'];save();print(json.dumps({k:record.get(k) for k in ['status','error','full_class_methods_retained','characters_per_runtime_leg']}))
sys.exit(0 if record['status']=='passed' else 1)
