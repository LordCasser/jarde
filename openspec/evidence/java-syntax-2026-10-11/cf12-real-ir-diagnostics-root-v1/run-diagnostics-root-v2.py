from pathlib import Path
import datetime,difflib,hashlib,importlib.util,json,os,re,shutil,subprocess,sys
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=Path(sys.argv[1]);OUT.mkdir(exist_ok=False,parents=True)
metadata=ROOT/'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/candidate-cli-v1.json'
assert hashlib.sha256(metadata.read_bytes()).hexdigest()=='9913bb51cfa4acc91ed10708f94acdcedb6b4eb85dd4ea31d827febbd05ad6ef'
md=json.loads(metadata.read_bytes()); pins={p:h for g in ('candidate_sources','test_sources','canonical_files') for p,h in md[g].items()}
assert len(pins)==52
for p,h in pins.items():assert hashlib.sha256((ROOT/p).read_bytes()).hexdigest()==h
paths=[ROOT/'crates/jarde-java/src/build.rs',ROOT/'crates/jarde-java/src/region.rs'];before={p:p.read_bytes() for p in paths}
for p,b in before.items():(OUT/(p.stem+'-product-before.rs')).write_bytes(b)
patch=Path('/private/tmp/jarde-cf12-local-types-real-ir-diagnostic-luna-v2/observer-and-runner.patch');fragment=Path('/private/tmp/jarde-cf12-real-ir-root-v1/diagnose_cf12_switch_ir.rs')
(OUT/'typed-local-private-v2.patch').write_bytes(patch.read_bytes());(OUT/'switch-private-v2.rs').write_bytes(fragment.read_bytes())
record={'schema':'cf12-real-ir-diagnostics-root-v2','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'commands':[],'source_pins_before':pins,'status':'running'}
def save():(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 with (OUT/'patch-check.stdout.raw').open('xb') as o,(OUT/'patch-check.stderr.raw').open('xb') as e:
  checked=subprocess.run(['git','apply','--check',str(patch)],cwd=ROOT,stdout=o,stderr=e)
 record['patch_check_exit_code']=checked.returncode;assert checked.returncode==0
 subprocess.run(['git','apply',str(patch)],cwd=ROOT,check=True)
 region=paths[1];s=region.read_text();at=s.rfind('\n}');assert at>=0;s=s[:at]+'\n'+fragment.read_text()+s[at:];region.write_text(s)
 for p in paths:
  (OUT/(p.stem+'-diagnostic.rs')).write_bytes(p.read_bytes())
  delta=''.join(difflib.unified_diff(before[p].decode().splitlines(True),p.read_text().splitlines(True),fromfile='a/'+p.relative_to(ROOT).as_posix(),tofile='b/'+p.relative_to(ROOT).as_posix()))
  (OUT/(p.stem+'-diagnostic-root.patch')).write_text(delta)
 guard=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';assert hashlib.sha256(guard.read_bytes()).hexdigest()=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
 spec=importlib.util.spec_from_file_location('cf12_guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);m.OUT=OUT;m.expected_test_summaries=lambda index,raw:None;m.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
 env=os.environ.copy()
 for key in m.STRIPPED_ENV_KEYS:env.pop(key,None)
 env.update(m.ENV_VALUES)
 for index,name in enumerate(['cf12_local_type_diagnostic_uses_frozen_class_ir_and_real_slot_uses','diagnose_cf12_switch_fallthrough_from_original_class_ir']):
  row=m.run_command(index,['cargo','test','-p','jarde-java','--lib',name,'--locked','--','--nocapture'],env);record['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None
  stream=Path(row['streams']['stdout']['path']);stream=stream if stream.is_absolute() else ROOT/stream;t=stream.read_text();assert re.search(r'(?m)^test (?:[A-Za-z0-9_]+::)*'+re.escape(name)+r' \.\.\. ok$',t);assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',t)==[('1','0','0')]
 record['status']='diagnostics-passed'
except Exception as exc:
 record.update(status='failed',error=f'{type(exc).__name__}: {exc}')
finally:
 for p,b in before.items():p.write_bytes(b)
 after={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in pins};record['source_pins_after']=after;record['all_52_pins_restored']=after==pins
 if after!=pins:record.update(status='failed',error='52 pins not restored')
 save();print(json.dumps({k:record.get(k) for k in ('status','error','all_52_pins_restored')}))
sys.exit(0 if record['status']=='diagnostics-passed' else 1)
