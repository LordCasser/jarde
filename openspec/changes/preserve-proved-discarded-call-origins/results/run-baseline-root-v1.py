from pathlib import Path
import importlib.util,json,os,hashlib,subprocess,sys,shutil,stat
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');HERE=Path(__file__).resolve().parent;INPUT=HERE/'original-inputs-root-v1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
p=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert sha(p)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
s=importlib.util.spec_from_file_location('probe_guard',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
m.OUT=HERE/'baseline-root-v1';m.OUT.mkdir(exist_ok=False);m.EXPECTED_SUMMARIES={};m.TEST_COMMANDS=set()
env=os.environ.copy()
for k in m.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(m.ENV_VALUES);env['LC_ALL']='C'
cli=Path('/private/tmp/jarde-return-arm-latch-cli-v1');assert sha(cli)=='9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f' and stat.S_IMODE(cli.stat().st_mode)==0o555
jadx=Path('/opt/homebrew/bin/jadx');assert sha(jadx)=='64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
rows=[];record={'schema':'discarded-call-source-baseline-root-v1','status':'running','commands':rows,'inputs':{n:sha(INPUT/n) for n in ['DiscardedCallSourceProbe.java','DiscardedCallSourceProbeRunner.java','ProbePop2.java']},'runner_sha256':sha(__file__),'cli':{'path':str(cli),'sha256':sha(cli)},'jadx':{'path':str(jadx),'sha256':sha(jadx)},'legs':{},'guards':{'free':m.FREE_LIMIT,'target':m.TARGET_LIMIT}}
def save(): (m.OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
def run(label,args):
 row=m.run_command(len(rows),[str(x) for x in args],env);row['label']=label;rows.append(row);save();print(json.dumps({'label':label,'exit':row['exit_code'],'peak':row['peak_target_bytes']}),flush=True);assert row['guard_stop'] is None;return row
try:
 for leg,home in [('javac8',Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home')),('javac23',Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'))]:
  d=m.OUT/leg;orig=d/'original';orig.mkdir(parents=True);tools={x:home/'bin'/x for x in ['java','javac','javap']};lr={'tools':{k:{'path':str(v),'sha256':sha(v)} for k,v in tools.items()},'profiles':{}};record['legs'][leg]=lr
  c=run(leg+'-original-compile',[tools['javac'],'-encoding','UTF-8','-g:none','-source','8','-target','8','-proc:none','-implicit:none','-d',orig,INPUT/'DiscardedCallSourceProbe.java',INPUT/'DiscardedCallSourceProbeRunner.java',INPUT/'ProbePop2.java']);assert c['exit_code']==0
  classes=sorted(orig.rglob('*.class'));lr['original_classes']={str(x.relative_to(orig)):sha(x) for x in classes}
  for name in ['DiscardedCallSourceProbe','ProbePop2']:
   assert run(leg+'-'+name+'-javap',[tools['javap'],'-p','-c','-v','-classpath',orig,'discardprobe.'+name])['exit_code']==0
  oracle=run(leg+'-original-runtime',[tools['java'],'-Xverify:all','-cp',orig,'discardprobe.DiscardedCallSourceProbeRunner']);assert oracle['exit_code']==0;lr['original_runtime_index']=oracle['index']
  cls=orig/'discardprobe/DiscardedCallSourceProbe.class'
  for profile in ['default','all','jadx']:
   out=d/profile;src=out/'sources';src.mkdir(parents=True)
   if profile=='jadx':
    rr=run(leg+'-jadx-decompile',[jadx,'--no-res','--config','none','--threads-count','1','-d',out/'decompile',cls]);assert rr['exit_code']==0
    gen=list((out/'decompile').rglob('DiscardedCallSourceProbe.java'));assert len(gen)==1;shutil.copyfile(gen[0],src/'DiscardedCallSourceProbe.java')
   else:
    args=[cli,'class-source','--input',cls,'--class','discardprobe.DiscardedCallSourceProbe','--policy','single-class','--release','8','--format','json']
    if profile=='all':args+=['--evidence','all']
    rr=run(leg+'-'+profile+'-render',args);assert rr['exit_code']==0;raw=Path(rr['streams']['stdout']['path']);raw=raw if raw.is_absolute() else ROOT/raw;doc=json.loads(raw.read_bytes());(out/'class.json').write_bytes(raw.read_bytes());(src/'DiscardedCallSourceProbe.java').write_text(doc['text'])
   shutil.copyfile(INPUT/'DiscardedCallSourceProbeRunner.java',src/'DiscardedCallSourceProbeRunner.java');compiled=out/'classes';compiled.mkdir()
   cc=run(leg+'-'+profile+'-compile',[tools['javac'],'-encoding','UTF-8','-g:none','-source','8','-target','8','-proc:none','-implicit:none','-classpath',compiled,'-d',compiled,src/'DiscardedCallSourceProbe.java',src/'DiscardedCallSourceProbeRunner.java'])
   pr={'render_index':rr['index'],'compile_index':cc['index'],'runtime_index':None};lr['profiles'][profile]=pr
   if cc['exit_code']==0:
    rt=run(leg+'-'+profile+'-runtime',[tools['java'],'-Xverify:all','-cp',compiled,'discardprobe.DiscardedCallSourceProbeRunner']);pr['runtime_index']=rt['index']
   save()
 record['status']='observed-baseline-not-accepted';save()
except Exception as e:record.update(status='failed',error=f'{type(e).__name__}: {e}');save();print(record['error'],file=sys.stderr)
sys.exit(0 if record['status']=='observed-baseline-not-accepted' else 1)
