import datetime,hashlib,json,os,signal,shutil,subprocess,time,traceback
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); JADX=Path('/Users/lordcasser/workspace/testzone/jadx'); OUT=Path('/private/tmp/jarde-cf12-direct-harness-root-v2'); OUT.mkdir(exist_ok=False)
SDK=Path('/private/tmp/jarde-cf12-test-sdk-root-v1'); JDK=Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'); API=JADX/'jadx-core/src/test/java/jadx'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def pin(p):return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
CASE=['TestSwitch','TestSwitchNoDefault','TestSwitchLabels','TestSwitchFallThrough','TestSwitchWithFallThroughCase']
SOURCE=[API/'NotYetImplemented.java',API/'NotYetImplementedExtension.java',API/'tests/api/IntegrationTest.java']
SOURCE+=sorted((API/'tests/api/compiler').glob('*.java'))
SOURCE+=[API/'tests/api/utils/TestFilesGetter.java',API/'tests/api/utils/TestUtils.java']
SOURCE+=sorted((API/'tests/api/utils/assertj').glob('*.java'))
SOURCE+=[API/f'tests/integration/switches/{c}.java' for c in CASE]
SOURCE+=[API/'api/JadxInternalAccess.java']
assert len(SOURCE)==24
PROD=sorted((JADX/'jadx-cli/build/install/jadx/lib').glob('*.jar')); JARS=sorted(SDK.glob('*.jar')); assert len(JARS)==6 and len(PROD)>20
for f in json.loads((SDK/'manifest.json').read_text())['artifacts']:
 for rec in f['files']:assert sha(Path(rec['path']))==rec['sha256']
for n,expected in {'java':'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1','javac':'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8'}.items():assert sha(JDK/'bin'/n)==expected
for n in ('fresh-classes','empty-sourcepath','junit-tmp','reports'): (OUT/n).mkdir()
for p in SOURCE:
 dst=OUT/'sources'/p.relative_to(JADX); dst.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(p,dst)
CP=os.pathsep.join(map(str,[OUT/'fresh-classes',*PROD,*JARS,JADX/'jadx-core/src/test/resources']))
env=os.environ.copy()
for key in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(key,None)
env['JAVA_HOME']=str(JDK); env['PATH']=str(JDK/'bin')+os.pathsep+env['PATH']
manifest={'schema':'cf12-fresh-official-upstream-harness-root-v2','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runner':pin(Path(__file__)),'source_files':[pin(p) for p in SOURCE],'product_runtime_jars':[pin(p) for p in PROD],'test_sdk_jars':[pin(p) for p in JARS],'jdk_tools':[pin(JDK/'bin'/n) for n in ('java','javac')],'environment':{k:env[k] for k in ('JAVA_HOME','PATH')},'cwd':str(JADX),'commands':[]}
def size(path):return sum(p.stat().st_size for p in path.rglob('*') if p.is_file()) if path.exists() else 0
def run(label,args,timeout=240):
 started=time.monotonic(); rec={'label':label,'argv':list(map(str,args)),'cwd':str(JADX),'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'guard':{'machine_floor_bytes':5*1024**3,'root_target_limit_bytes':1024**3,'private_output_limit_bytes':1024**3,'poll_seconds':1,'abort':None,'peak_output_bytes':0,'peak_target_bytes':0}}
 with (OUT/(label+'.stdout.raw')).open('wb') as o,(OUT/(label+'.stderr.raw')).open('wb') as e:
  p=subprocess.Popen(rec['argv'],cwd=JADX,env=env,stdout=o,stderr=e,start_new_session=True)
  while p.poll() is None:
   free=shutil.disk_usage(ROOT).free; target=size(ROOT/'target'); output=size(OUT)
   rec['guard']['peak_output_bytes']=max(output,rec['guard']['peak_output_bytes']);rec['guard']['peak_target_bytes']=max(target,rec['guard']['peak_target_bytes'])
   reason='disk_floor' if free<5*1024**3 else 'target_limit' if target>1024**3 else 'output_limit' if output>1024**3 else 'timeout' if time.monotonic()-started>timeout else None
   if reason:
    rec['guard']['abort']=reason; os.killpg(p.pid,signal.SIGTERM)
    try:p.wait(timeout=5)
    except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
    break
   time.sleep(1)
  rec['exit_code']=p.wait()
 rec['duration_seconds']=time.monotonic()-started;rec['streams']=[pin(OUT/(label+'.'+n+'.raw')) for n in ('stdout','stderr')]; manifest['commands'].append(rec)
 (OUT/'execution.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print(json.dumps({'label':label,'exit_code':rec['exit_code'],'abort':rec['guard']['abort'],'seconds':rec['duration_seconds']}),flush=True)
 assert rec['exit_code']==0 and rec['guard']['abort'] is None,label
try:
 run('javac-version',[JDK/'bin/javac','-version'])
 run('fresh-compile',[JDK/'bin/javac','-encoding','UTF-8','-source','11','-target','11','-proc:none','-implicit:none','-sourcepath',OUT/'empty-sourcepath','-classpath',CP,'-d',OUT/'fresh-classes',*SOURCE])
 run('upstream-junit',[JDK/'bin/java',f'-Duser.dir={JADX}',f'-Djava.io.tmpdir={OUT}/junit-tmp','-cp',CP,'org.junit.platform.console.ConsoleLauncher','execute','--disable-ansi-colors','--disable-banner','--fail-if-no-tests','--details=tree',f'--reports-dir={OUT}/reports',f'--redirect-stdout={OUT}/test-stdout.raw',f'--redirect-stderr={OUT}/test-stderr.raw','--config=junit.jupiter.execution.parallel.enabled=false','--config=junit.jupiter.tempdir.cleanup.mode.default=NEVER',*[f'--select-class=jadx.tests.integration.switches.{c}' for c in CASE]])
 manifest['result']='commands_succeeded_pending_independent_acceptance'
except Exception:
 manifest['result']='failed';manifest['error']=traceback.format_exc();raise
finally:
 manifest['source_unchanged']=all(sha(Path(p['path']))==p['sha256'] for p in manifest['source_files']);manifest['generated_files']=[pin(p) for p in OUT.rglob('*') if p.is_file() and p.name!='execution.json'];manifest['free_bytes_after']=shutil.disk_usage(ROOT).free
 (OUT/'execution.json').write_text(json.dumps(manifest,indent=2)+'\n')
