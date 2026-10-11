import hashlib,importlib.util,json,os,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); OUT=Path(sys.argv[1]);OUT.mkdir(exist_ok=False)
SHA=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert SHA(GUARD)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
s=importlib.util.spec_from_file_location('guard',GUARD);g=importlib.util.module_from_spec(s);s.loader.exec_module(g)
scanpath=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py';assert SHA(scanpath)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
s=importlib.util.spec_from_file_location('scan',scanpath);scan=importlib.util.module_from_spec(s);s.loader.exec_module(scan)
g.OUT=OUT;g.target_bytes=lambda:scan.target_bytes(ROOT);g.expected_test_summaries=lambda i,b:None
g.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':SHA(p)}
env=os.environ.copy();env.update(g.ENV_VALUES)
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
plan=ROOT/'openspec/changes/recover-proved-switch-continuations/results/replay-planning-root-v1/plan-luna-v1.json'
assert SHA(plan)=='f8c0ee7b9b87a9872ce86e38614fce590a47d33b23f40af1d8cf07a2e4d4cb9c'
p=json.loads(plan.read_text())
rec={'schema':'compound-real-bytecode-jvm-verifier-root-v1','status':'running','runner_sha256':SHA(__file__),'plan_sha256':SHA(plan),'commands':[],'fixtures':[],'runtime_scope':'Class.forName + getDeclaredMethods under -Xverify:all; altered test is never invoked'}
try:
 for source in map(Path,sys.argv[2:]):
  name=source.stem;folder=OUT/name;folder.mkdir();target=folder/'CompoundStatementConditions.class';shutil.copyfile(source,target)
  driver=folder/'VerifyOnly.java';driver.write_text('public final class VerifyOnly { public static void main(String[] args) throws Exception { Class<?> type = Class.forName("CompoundStatementConditions"); System.out.println("verified:" + type.getName() + ":methods=" + type.getDeclaredMethods().length); } }\n')
  row={'source':str(source),'source_sha256':SHA(source),'class_bytes':target.stat().st_size,'copied_class':str(target),'copied_class_sha256':SHA(target),'driver_sha256':SHA(driver),'jdk':[]};rec['fixtures'].append(row)
  for jdk in ['jdk8_tools','jdk23_tools']:
   tools=p['shared_pins'][jdk]
   for n in ['java','javac','javap']:
    assert SHA(tools[n]['path'])==tools[n]['sha256']
   leg_env=env.copy();leg_env['JAVA_HOME']=str(Path(tools['java']['path']).resolve().parent.parent)
   classes=folder/jdk;classes.mkdir()
   commands=[['javap','-v','-p','-c',str(target)],['javac','-encoding','UTF-8','-proc:none','-source','8','-target','8','-d',str(classes),str(driver)],['java','-Xverify:all','-cp',str(classes)+os.pathsep+str(folder),'VerifyOnly']]
   for argv in commands:
    argv[0]=tools[argv[0]]['path'];idx=len(rec['commands']);item=g.run_command(idx,argv,leg_env);rec['commands'].append(item)
    assert item['exit_code']==0 and item['guard_stop'] is None,(jdk,name,argv,item['exit_code'])
    if argv[-1]=='VerifyOnly':assert (OUT/f'{idx}.stdout.raw').read_bytes()==b'verified:CompoundStatementConditions:methods=7\n'
   assert SHA(target)==row['source_sha256'];row['jdk'].append({'name':jdk,'verified':True,'tools':tools,'driver_classes':[str(f.relative_to(classes)) for f in classes.rglob('*.class')]});assert row['jdk'][-1]['driver_classes']==['VerifyOnly.class']
 rec['status']='all-jvm-verifiers-accepted'
except BaseException as e:rec.update(status='failed',error=repr(e))
finally:
 (OUT/'execution.json').write_text(json.dumps(rec,indent=2)+'\n');print(json.dumps({k:rec.get(k) for k in ['status','error','runtime_scope']}),flush=True)
sys.exit(0 if rec['status']=='all-jvm-verifiers-accepted' else 1)
