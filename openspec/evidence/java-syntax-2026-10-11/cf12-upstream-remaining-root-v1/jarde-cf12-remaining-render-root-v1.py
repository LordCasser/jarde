import datetime,hashlib,json,os,shutil,signal,stat,subprocess,time
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); BASE=Path('/private/tmp/jarde-cf12-remaining-direct-root-v1'); OUT=Path('/private/tmp/jarde-cf12-remaining-render-root-v1');OUT.mkdir(exist_ok=False)
CLI=Path('/private/tmp/jarde-proved-conditional-switch-cli-v1');JAVAP=Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(CLI)=='39d5699c1665b16e0c4a46934f0e773aeedf392f5bc60869a7b2762680dd10f1' and CLI.stat().st_mode&0o777==0o555
record={'schema':'cf12-remaining-five-frozen-conditional-cli-render-root-v1','runner':{'path':str(Path(__file__)),'sha256':sha(Path(__file__))},'cli_product_commit':'6476b56c357443ef17318891b12142f509977234','cli':{'path':str(CLI),'sha256':sha(CLI)},'harness_execution':{'path':str(BASE/'execution.json'),'sha256':sha(BASE/'execution.json')},'commands':[]}
env=os.environ.copy()
for k in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(k,None)
def size(path):
 total=0
 for p in path.rglob('*'):
  try: info=p.stat()
  except FileNotFoundError: continue
  if stat.S_ISREG(info.st_mode): total+=info.st_size
 return total
def run(d,label,args):
 start=time.monotonic();rec={'label':label,'argv':list(map(str,args)),'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'guard_stop':None,'cwd':str(ROOT)}
 with (d/(label+'.stdout.raw')).open('wb') as o,(d/(label+'.stderr.raw')).open('wb') as e:
  p=subprocess.Popen(rec['argv'],cwd=ROOT,env=env,stdout=o,stderr=e,start_new_session=True)
  while p.poll() is None:
   reason='free_floor' if shutil.disk_usage(ROOT).free<5*1024**3 else 'target_limit' if size(ROOT/'target')>1024**3 else 'output_limit' if size(OUT)>1024**3 else 'timeout' if time.monotonic()-start>60 else None
   if reason:
    rec['guard_stop']=reason;os.killpg(p.pid,signal.SIGTERM)
    try:p.wait(timeout=5)
    except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
    break
   time.sleep(1)
  rec['exit_code']=p.wait()
 rec['seconds']=time.monotonic()-start;rec['streams']=[{'path':str(d/(label+'.'+s+'.raw')),'bytes':(d/(label+'.'+s+'.raw')).stat().st_size,'sha256':sha(d/(label+'.'+s+'.raw'))} for s in ('stdout','stderr')];record['commands'].append(rec)
 (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
 return rec
assert shutil.disk_usage(ROOT).free>=5*1024**3 and size(ROOT/'target')<=1024**3
for case in sorted((BASE/'capture').iterdir()):
 for file in sorted((case/'input').glob('*.class')):
  name='jadx.tests.integration.switches.'+file.stem;d=OUT/case.name/file.stem;d.mkdir(parents=True)
  assert run(d,'javap',[JAVAP,'-p','-c','-s','-v',file])['exit_code']==0
  record.setdefault('inputs',[]).append({'path':str(file),'bytes':file.stat().st_size,'sha256':sha(file)})
  for mode in ('default','all'):
   args=[CLI,'class-source','--input',file,'--class',name,'--policy','single-class','--release','8','--format','json']
   if mode=='all':args+=['--evidence','all']
   rec=run(d,'jarde-'+mode,args)
   if rec['exit_code']==0:
    doc=json.loads((d/('jarde-'+mode+'.stdout.raw')).read_text());(d/('jarde-'+mode+'.java')).write_text(doc.get('text',''))
   print(json.dumps({'case':case.name,'class':file.stem,'mode':mode,'exit':rec['exit_code']}),flush=True)
record['status']='observed-remaining-five-render-pending-independent-review'
record['free_bytes_after']=shutil.disk_usage(ROOT).free
(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
