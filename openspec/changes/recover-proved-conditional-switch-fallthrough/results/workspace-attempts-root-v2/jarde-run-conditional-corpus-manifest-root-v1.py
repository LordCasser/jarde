from pathlib import Path
import hashlib, importlib.util, json, os, re, runpy, sys
sys.dont_write_bytecode=True
root=Path('/Users/lordcasser/workspace/projects/jarde')
out=Path('/private/tmp/jarde-conditional-corpus-manifest-root-v1');out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
scan_path=root/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
assert sha(guard)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(scan_path)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
spec=importlib.util.spec_from_file_location('manifest_guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
scan=runpy.run_path(str(scan_path),run_name='manifest_size_scan')
m.ROOT=root;m.OUT=out;m.target_bytes=lambda:scan['target_bytes'](root)
m.expected_test_summaries=lambda *_:None
m.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
m.ENV_VALUES={**m.ENV_VALUES,'PROPTEST_RNG_SEED':'5350648285461741569','CARGO_PROFILE_TEST_STRIP':'symbols'}
env=os.environ.copy()
for key in m.STRIPPED_ENV_KEYS:env.pop(key,None)
env.update(m.ENV_VALUES)
pins=json.loads(Path('/private/tmp/jarde-conditional-workspace-batches-root-v3/execution.json').read_bytes())['source_pins']
assert all(sha(root/p)==h for p,h in pins.items())
manifest=root/'tests/fixtures/corpus-fingerprint.json'
before=manifest.read_bytes();(out/'corpus-fingerprint.before.json').write_bytes(before)
record={'schema':'conditional-corpus-manifest-refresh-root-v1','status':'running','source_pins':pins,'manifest_before_sha256':hashlib.sha256(before).hexdigest(),'guard_template':{'path':str(guard),'sha256':sha(guard)},'target_size_scan_adapter':{'path':str(scan_path),'sha256':sha(scan_path)},'commands':[]}
save=lambda:(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n');save()
try:
 argv=['cargo','test','-p','jarde','--test','p5_corpus_fingerprint','--all-features','--locked','--','--ignored','--nocapture','regenerate_corpus_fingerprint']
 row=m.run_command(0,argv,env);record['commands'].append(row);save()
 assert row['exit_code']==0 and row['guard_stop'] is None
 assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', (out/'0.stdout.raw').read_text())==[('1','0','0')]
 after=manifest.read_bytes();(out/'corpus-fingerprint.after.json').write_bytes(after)
 a=json.loads(before);b=json.loads(after)
 assert {k:v for k,v in a.items() if k!='files'}=={k:v for k,v in b.items() if k!='files'}
 old={r['path']:r for r in a['files']};new={r['path']:r for r in b['files']}
 assert all(new.get(p)==v for p,v in old.items())
 added=sorted(set(new)-set(old));expected=sorted(p.relative_to(root).as_posix() for p in (root/'tests/fixtures/p3-conditional-switch-boundaries').rglob('*.class'))
 assert len(added)==6 and added==expected
 record.update(status='passed',added_files=[new[p] for p in added],manifest_after_sha256=hashlib.sha256(after).hexdigest(),before_file_count=len(old),after_file_count=len(new))
except BaseException as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:
 record['source_pins_unchanged']=all(sha(root/p)==h for p,h in pins.items());save();print(json.dumps({k:record.get(k) for k in ('status','error','source_pins_unchanged','before_file_count','after_file_count','added_files')}))
sys.exit(0 if record['status']=='passed' and record['source_pins_unchanged'] else 1)
