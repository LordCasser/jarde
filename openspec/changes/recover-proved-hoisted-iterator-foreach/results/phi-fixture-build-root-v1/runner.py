import hashlib,importlib.util,json,os,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-hoisted-iterator-phi-fixture-build-root-v1');S=Path('/private/tmp/jarde-hoisted-iterator-regression-prep-luna-v3/HoistedIteratorPhiControls.java')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(name,path,pin):
 assert H(path)==pin;s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
assert H(S)=='9cdf3915818be01928664d3c112b3f846797b9f71d110dc49df8d8144bd59bbb';assert not O.exists() and not (R/'target').exists();O.mkdir();(O/'raw').mkdir();shutil.copyfile(S,O/S.name);(O/S.name).chmod(0o444)
g=load('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33');scan=load('scanner',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7');g.ROOT=R;g.OUT=O/'raw';g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,r:None;g.ENV_VALUES={};g.command_stream=lambda p:dict(path=str(p),bytes=Path(p).stat().st_size,sha256=H(p))
jm=R/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json';assert H(jm)=='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec';j=json.loads(jm.read_bytes());e=os.environ.copy()
for k in g.STRIPPED_ENV_KEYS:e.pop(k,None)
e.pop('RUST_MIN_STACK',None)
x={'schema':'hoisted-iterator-phi-fixture-build-root-v1','status':'running','source_sha256':H(S),'jdk_manifest_sha256':H(jm),'runner_sha256':H(__file__),'commands':[],'classes':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
for leg in j['legs']:
 name=leg['leg'];tools=leg['jdk_tools'];home=str(Path(tools['java']['path']).parent.parent);env=e.copy();env['JAVA_HOME']=home;env['JDK_HOME']=home;dest=O/name;dest.mkdir();empty=dest/'empty';empty.mkdir();classes=dest/'classes';classes.mkdir()
 for t in tools.values():assert H(t['path'])==t['sha256']
 flags=['-source','8','-target','8','-Xlint:-options'] if name=='javac8' else ['--release','8']
 for argv in [[tools['javac']['path'],*flags,'-proc:none','-g:none','-classpath',str(empty),'-sourcepath',str(empty),'-d',str(classes),str(O/S.name)],[tools['javap']['path'],'-p','-c','-s','-v','-classpath',str(classes),'cf11.HoistedIteratorPhiControls']]:
  c=g.run_command(len(x['commands']),argv,env);x['commands'].append(c);save();assert c['exit_code']==0 and c['guard_stop'] is None
 f=classes/'cf11/HoistedIteratorPhiControls.class';f.chmod(0o444);x['classes'].append(dict(jdk=name,path=str(f),bytes=f.stat().st_size,sha256=H(f)))
assert len(x['classes'])==2 and not (R/'target').exists() and H(S)==x['source_sha256'];x.update(status='compiled_fixture_only',target_absent=True,dual_jdk_identical=x['classes'][0]['sha256']==x['classes'][1]['sha256']);save();print(json.dumps({k:v for k,v in x.items() if k!='commands'}))
