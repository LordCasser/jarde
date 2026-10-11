import pathlib,json,hashlib,os,sys,importlib.util,shutil
sys.dont_write_bytecode=True
ROOT=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
OUT=pathlib.Path('/private/tmp/jarde-computed-for-call-order-candidate-root-v1');OUT.mkdir()
SRC=pathlib.Path('/private/tmp/jarde-computed-for-call-order-control-luna-v2')
CLI=pathlib.Path('/private/tmp/jarde-computed-for-candidate-cli-root-v1')
def pin(p):
 p=pathlib.Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
guard=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
spec=importlib.util.spec_from_file_location('guard',guard);g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
scanner=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
s=importlib.util.spec_from_file_location('scan',scanner);scan=importlib.util.module_from_spec(s);s.loader.exec_module(scan)
g.OUT=OUT/'raw';g.OUT.mkdir();g.command_stream=pin;g.target_bytes=lambda:scan.target_bytes(ROOT)
g.EXPECTED_SUMMARIES={};g.TEST_COMMANDS=set()
env=os.environ.copy()
for key in ('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(key,None)
env.update(g.ENV_VALUES)
rec={'status':'running','runner':pin(__file__),'guard':pin(guard),'scanner':pin(scanner),'source_pins':[pin(SRC/n) for n in ('ComputedForEffectControl.java','Runner.java','swap_init_store_with_g.py')],'cli':pin(CLI),'commands':[],'profiles':[]}
build_path=pathlib.Path('/private/tmp/jarde-computed-for-cli-build-root-v1/execution.json')
build=json.loads(build_path.read_text());assert build['status']=='validation-passed-cli-frozen-target-cleaned'
frozen=build['freeze']['frozen_cli'];assert rec['cli']['path']==frozen['path'] and rec['cli']['sha256']==frozen['sha256'] and rec['cli']['bytes']==frozen['size_bytes']
rec['build_receipt']=pin(build_path)
def save(): (OUT/'execution.json').write_text(json.dumps(rec,indent=2)+'\n')
def run(argv):
 row=g.run_command(100+len(rec['commands']),list(map(str,argv)),env);rec['commands'].append(row);save()
 if row['exit_code'] or row['guard_stop']:raise RuntimeError(str(argv))
 return row
try:
 for leg,home in [('jdk8','/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),('jdk23','/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
  b=OUT/leg;b.mkdir();original=b/'original';pending=b/'pending';original.mkdir();pending.mkdir()
  tools={n:pathlib.Path(home)/'bin'/n for n in ('javac','java','javap')}
  prof={'jdk':leg,'tools':{n:pin(p) for n,p in tools.items()},'renders':[],'runtimes':{}}
  rec['profiles'].append(prof);save()
  run([tools['javac'],'-encoding','UTF-8','-g:none','-source','8','-target','8','-proc:none','-d',original,SRC/'ComputedForEffectControl.java',SRC/'Runner.java'])
  run([sys.executable,SRC/'swap_init_store_with_g.py','--input',original/'ComputedForEffectControl.class','--output',pending/'ComputedForEffectControl.class'])
  shutil.copy2(original/'Runner.class',pending/'Runner.class')
  prof['original_class']=pin(original/'ComputedForEffectControl.class');prof['pending_class']=pin(pending/'ComputedForEffectControl.class')
  for variant,classes in [('original',original),('pending',pending)]:
   run([tools['javap'],'-classpath',classes,'-c','-v','ComputedForEffectControl'])
   row=run([tools['java'],'-Xverify:all','-cp',classes,'Runner']);prof['runtimes'][variant]=row
   for mode in ('default','all'):
    argv=[CLI,'class-source','--input',classes/'ComputedForEffectControl.class','--class','ComputedForEffectControl','--policy','single-class','--release','8','--format','json']
    if mode=='all':argv+=['--evidence','all']
    render=run(argv);doc=json.load(open(render['streams']['stdout']['path']))
    d=b/(variant+'-'+mode);d.mkdir();src=d/'ComputedForEffectControl.java';src.write_bytes(doc['text'].encode())
    run([tools['javac'],'-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',d,src])
    row=run([tools['java'],'-Xverify:all','-cp',str(d)+os.pathsep+str(original),'Runner'])
    baseline=pathlib.Path(prof['runtimes'][variant]['streams']['stdout']['path']).read_bytes()
    assert pathlib.Path(row['streams']['stdout']['path']).read_bytes()==baseline
    prof['renders'].append({'variant':variant,'mode':mode,'report':render['streams']['stdout'],'source':pin(src),'runtime':row,'matches_original_variant':True})
  assert pathlib.Path(prof['runtimes']['original']['streams']['stdout']['path']).read_bytes()==pathlib.Path(prof['runtimes']['pending']['streams']['stdout']['path']).read_bytes()
 rec['status']='observed_complete_pending_root_review';save()
except BaseException as ex:
 rec['status']='failed';rec['failure']=type(ex).__name__+': '+str(ex);save();raise
