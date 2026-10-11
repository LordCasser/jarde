import importlib.util,json,hashlib,os,sys,shutil,difflib
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-hoisted-iterator-gateway-runtime-root-v1');assert not O.exists();O.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(n,p,h):
 assert H(p)==h;s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33');g.ROOT=R;g.OUT=O;g.expected_test_summaries=lambda i,r:None;g.command_stream=lambda p:dict(path=str(p),bytes=Path(p).stat().st_size,sha256=H(p))
scan=load('scan',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7');g.target_bytes=lambda:scan.target_bytes(R)
fixture=R/'openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/baseline-root-v1/cases/javac23-original/classes/VariablePostfixLoop.class'
golden=R/'openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/baseline-root-v1/cases/javac23-jarde-render/class-source-default.json'
candidate=Path('/private/tmp/jarde-hoisted-iterator-unicode-regression-prep-luna-v1/SB.jarde.java.candidate')
x={'schema':'hoisted-iterator-gateway-runtime-root-v1','status':'running','commands':[],'runner_sha256_before':H(__file__),'input_sha256':H(fixture),'historical_capture_sha256':H(golden),'comparisons':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv,home=None):
 env=os.environ.copy();env.pop('RUST_MIN_STACK',None)
 for k in ('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(k,None)
 if home:env['JAVA_HOME']=str(home);env['PATH']=str(home/'bin')+os.pathsep+env['PATH']
 row=g.run_command(len(x['commands']),[str(v) for v in argv],env);x['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None;return tuple(Path(row['streams'][k]['path']).read_bytes() for k in ('stdout','stderr'))
try:
 assert not (R/'target').exists()
 reports={}
 for tag,cli,evidence in [('old',Path('/private/tmp/jarde-constructor-conditional-final-cli-root-v1'),None),('new-default',Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2'),None),('new-all',Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2'),'all')]:
  assert H(cli)==('a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241' if tag=='old' else '8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a')
  argv=[cli,'class-source','--input',fixture,'--class','VariablePostfixLoop','--policy','single-class','--release','8','--format','json']
  if evidence:argv+=['--evidence',evidence]
  raw,_=run(argv);doc=json.loads(raw);(O/(tag+'.json')).write_bytes(raw);reports[tag]=doc
 print('report keys',list(reports['new-default']),flush=True)
 # The report envelope is the CLI's same-run JSON result.
 def text(d):
  if 'text' in d:return d['text']
  for k in ('data','result','report'):
   if isinstance(d.get(k),dict):
    v=text(d[k])
    if v is not None:return v
  return None
 texts={k:text(v) for k,v in reports.items()};assert all(isinstance(v,str) for v in texts.values()),list(reports['new-default'])
 assert texts['new-default']==texts['new-all']
 # The source diff is independently reviewed before applying a test expectation update.
 (O/'source.diff').write_text(''.join(difflib.unified_diff(texts['old'].splitlines(True),texts['new-default'].splitlines(True),fromfile='old/VariablePostfixLoop.java',tofile='new/VariablePostfixLoop.java')))
 driver=(R/'openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/baseline-root-v1/cases/javac23-original/Runner.java').read_text();x['driver_sha256']=hashlib.sha256(driver.encode()).hexdigest()
 jm=R/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json';assert H(jm)=='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
 for leg in json.loads(jm.read_text())['legs']:
  if leg['leg'] not in ('javac8','javac23'):continue
  tools=leg['jdk_tools'];home=Path(tools['java']['path']).parent.parent
  for v in tools.values():assert H(v['path'])==v['sha256']
  baseline=None
  for tag in ('original','old','new-default'):
   w=O/leg['leg']/tag;w.mkdir(parents=True);classes=w/'classes';classes.mkdir();empty=w/'empty';empty.mkdir();runner=w/'Runner.java';runner.write_text(driver)
   flags=['-source','8','-target','8','-Xlint:-options'] if leg['leg']=='javac8' else ['--release','8']
   if tag=='original':
    shutil.copyfile(fixture,classes/'VariablePostfixLoop.class');sources=[runner];classpath=classes
   else:
    source=w/'VariablePostfixLoop.java';source.write_text(texts[tag]);sources=[source,runner];classpath=empty
   run([tools['javac']['path'],*flags,'-proc:none','-classpath',classpath,'-sourcepath',empty,'-d',classes,*sources],home)
   stdout,stderr=run([tools['java']['path'],'-Xverify:all','-cp',classes,'Runner'],home)
   if tag=='original':baseline=(stdout,stderr)
   assert (stdout,stderr)==baseline,(leg['leg'],tag,stdout,stderr)
   x['comparisons'].append({'jdk':leg['leg'],'tag':tag,'stdout_sha256':hashlib.sha256(stdout).hexdigest(),'stderr_sha256':hashlib.sha256(stderr).hexdigest(),'same_original':True});save()
 x.update(status='passed_fixed_gateway_cases_only',target_absent=not (R/'target').exists(),runner_sha256_after=H(__file__));assert x['runner_sha256_after']==x['runner_sha256_before'];save();print(json.dumps({'status':x['status'],'commands':len(x['commands']),'comparisons':len(x['comparisons']),'target_absent':x['target_absent']}))
except BaseException as e:x.update(status='failed',error=repr(e));save();raise
