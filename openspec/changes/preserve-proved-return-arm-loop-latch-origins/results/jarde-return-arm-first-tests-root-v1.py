from pathlib import Path
import hashlib,importlib.util,json,os
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');OUT=Path('/private/tmp/jarde-return-arm-first-tests-root-v1');OUT.mkdir(exist_ok=False)
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(TEMPLATE)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
spec=importlib.util.spec_from_file_location('guard',TEMPLATE);guard=importlib.util.module_from_spec(spec);spec.loader.exec_module(guard);guard.OUT=OUT
# Raw log paths remain the actual private execution paths.
guard.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
guard.expected_test_summaries=lambda i,raw:{'actual':[tuple(map(int,x)) for x in guard.SUMMARY_RE.findall(raw.decode())],'expected':[(15,0,0)],'ok': [tuple(map(int,x)) for x in guard.SUMMARY_RE.findall(raw.decode())]==[(15,0,0)]}
env=os.environ.copy()
for k in guard.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(guard.ENV_VALUES);jdk=guard.configure_jdk23(env)
record={'schema':'return-arm-latch-first-permanent-tests-root-v1','runner_sha256':sha(Path(__file__)),'guard_template_sha256':sha(TEMPLATE),'jdk':jdk,'commands':[],'product_pins':{str(p.relative_to(ROOT)):sha(p) for p in [ROOT/'crates/jarde-java/src/region.rs',ROOT/'crates/jarde-java/tests/p3_loop_exit_gateways.rs']}}
row=guard.run_command(2,['cargo','test','-p','jarde-java','--test','p3_loop_exit_gateways','--locked','--','--nocapture'],env);record['commands'].append(row)
record['status']='passed' if row['exit_code']==0 and row['guard_stop'] is None and row['test_summary_check']['ok'] else 'failed';(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print({'status':record['status'],'target_peak':row.get('target_bytes_peak')});raise SystemExit(0 if record['status']=='passed' else 1)
