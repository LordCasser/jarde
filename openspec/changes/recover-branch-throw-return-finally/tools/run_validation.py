"""Run one root-controlled guarded command sequence; always perform exact Cargo clean."""
import datetime,hashlib,importlib.util,json,os,re,shutil,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
OUT=Path(sys.argv[1]).resolve();SPEC=Path(sys.argv[2]).resolve()
config=json.loads(SPEC.read_text());commands=config if isinstance(config,list) else config['commands'];allowed=set() if isinstance(config,list) else set(config.get('allowed_source_changes',[]));assert allowed <= {'tests/fixtures/corpus-fingerprint.json'}
assert not OUT.exists() and not (ROOT/'target').exists()
assert shutil.disk_usage(ROOT).free>=5*1024**3
OUT.mkdir();(OUT/'raw').mkdir()
def pin(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def module(path,name):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCANNER=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
assert pin(GUARD)['sha256']=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert pin(SCANNER)['sha256']=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
g=module(GUARD,'guard');s=module(SCANNER,'scanner');g.ROOT=ROOT;g.OUT=OUT/'raw';g.target_bytes=lambda:s.target_bytes(ROOT);g.command_stream=pin;g.expected_test_summaries=lambda i,b:None
values={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'never'}
env={k:v for k,v in os.environ.items() if k not in ('RUST_MIN_STACK','CARGO_TARGET_DIR','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')};env.update(values);g.ENV_VALUES=values
# The current pinned Java installation is also used by ignored comparison tests when requested.
env['JAVA_HOME']='/Users/lordcasser/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home';env['PATH']=env['JAVA_HOME']+'/bin'+os.pathsep+env.get('PATH','')
def sources():
 raw=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z','--','src','crates','tests','Cargo.toml','Cargo.lock'],cwd=ROOT)
 return [pin(ROOT/os.fsdecode(p)) for p in sorted(set(raw.split(b'\0'))-{b''}) if (ROOT/os.fsdecode(p)).is_file()]
r={'status':'running','runner':pin(__file__),'spec':pin(SPEC),'guard':pin(GUARD),'scanner':pin(SCANNER),'source_before':sources(),'default_stack':True,'commands':[]}
def save(): (OUT/'execution.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
save()
try:
 for argv in commands:
  row=g.run_command(len(r['commands'])+1,argv,env);r['commands'].append(row);save()
  assert row['exit_code']==0 and row['guard_stop'] is None, 'command failed or guard stopped'
  if argv[:2]==['cargo','test']:
   raw=Path(row['streams']['stdout']['path']).read_text();summaries=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)
   row['test_summaries']=summaries;save();assert summaries and all(x[0]=='ok' and int(x[1])>0 and x[2]=='0' for x in summaries), 'missing or unsuccessful tests'
  if argv[:2]==['cargo','test']:
   row['removed_successful_test_artifacts']=[];g.clean_successful_test_binary(row['index'],row['removed_successful_test_artifacts']);save()
  if argv[:2]==['cargo','clippy']:
   row['removed_check_only_metadata']=[];g.clean_check_only_metadata(row['removed_check_only_metadata']);save()
  if argv[:2]==['cargo','build']:
   cli=OUT/'candidate-cli';shutil.copyfile(ROOT/'target/debug/jarde-cli',cli);cli.chmod(0o555);r['cli']=pin(cli)
 r['status']='commands_passed_pending_independent_acceptance'
except BaseException as e: r.update(status='failed',error=str(e))
finally:
 r['source_after']=sources();r['source_unchanged']=r['source_before']==r['source_after']
 before={p['path']:p for p in r['source_before']};after={p['path']:p for p in r['source_after']};changed={str(Path(p).relative_to(ROOT)) for p in before.keys()|after.keys() if before.get(p)!=after.get(p)};r['source_changes']=sorted(changed);r['allowed_source_changes']=sorted(allowed);r['source_changes_accepted']=changed==allowed
 i=len(r['commands'])+1;so=OUT/'raw'/f'{i}.stdout.raw';se=OUT/'raw'/f'{i}.stderr.raw'
 with so.open('xb') as a,se.open('xb') as b: rc=subprocess.run(['cargo','clean'],cwd=ROOT,env=env,stdout=a,stderr=b).returncode
 r['commands'].append({'index':i,'argv':['cargo','clean'],'exit_code':rc,'streams':{'stdout':pin(so),'stderr':pin(se)},'cleanup_only':True});r['target_absent']=not (ROOT/'target').exists()
 if rc!=0 or not r['target_absent'] or not r['source_changes_accepted']:r['status']='failed'
 r['finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save();print(json.dumps({k:r.get(k) for k in ('status','error','target_absent','source_unchanged','cli')}))
sys.exit(0 if r['status']=='commands_passed_pending_independent_acceptance' else 1)
