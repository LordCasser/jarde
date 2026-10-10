from pathlib import Path
import datetime,hashlib,importlib.util,json,os,re,sys
root=Path('/Users/lordcasser/workspace/projects/jarde')
out=Path('/private/tmp/jarde-typed-ci-p5-billing-record-root-v1');out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
md=json.loads((root/'openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json').read_text())
pins={k:v for group in ('candidate_sources','test_sources','canonical_files') for k,v in md[group].items()}
for path,digest in pins.items():assert sha(root/path)==digest,path
additional={p:sha(root/p) for p in ['tests/p3_meeting.rs','tests/fixtures/p3-meeting/v8/Meet.class','tests/fixtures/p3-meeting/Meet.java','tests/p3_required_conversions.rs','tests/p3_reference_slot_lifetimes.rs','tests/fixtures/p3-required-conversions/RequiredConversions.java','tests/fixtures/p3-required-conversions/v8/RequiredConversions.class','tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.class','tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.baseline.java']}
guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert sha(guard)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
spec=importlib.util.spec_from_file_location('repair_guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
g=m.run_command.__globals__;g['ROOT']=root;g['OUT']=out;g['expected_test_summaries']=lambda index,raw:None;g['command_stream']=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
env=os.environ.copy()
for key in m.STRIPPED_ENV_KEYS:env.pop(key,None)
env.update(m.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env['CARGO_PROFILE_TEST_STRIP']='symbols'
record={'schema':'typed-p5-billing-record-root-v1','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_pins':pins,'additional_test_pins':additional,'commands':[],'status':'running'}
save=lambda:(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
save()
commands=[['cargo','test','--test','p5_bulk_corpus','--all-features','--locked','--message-format','json','--','--ignored','--nocapture','record_the_billing_table']]
try:
 for index,argv in enumerate(commands):
  row=m.run_command(index,argv,env);record['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None,(argv,row['exit_code'],row['guard_stop'])
  assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', (out/'0.stdout.raw').read_text())==[('1','0','0')]
 record['status']='passed'
except BaseException as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:
 record['pins_unchanged']=all(sha(root/p)==h for p,h in {**pins,**additional}.items());save();print(json.dumps({k:record.get(k) for k in ['status','error','workspace_summary_count','workspace_totals','pins_unchanged']}))
sys.exit(0 if record['status']=='passed' else 1)
