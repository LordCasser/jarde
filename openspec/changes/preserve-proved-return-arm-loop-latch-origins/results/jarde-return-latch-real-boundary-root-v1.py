from pathlib import Path
import hashlib,importlib.util,json,os,subprocess
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');OUT=Path('/private/tmp/jarde-return-latch-real-boundary-root-v1');OUT.mkdir(exist_ok=False)
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(TEMPLATE)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
region=ROOT/'crates/jarde-java/src/region.rs';original=region.read_bytes();(OUT/'region-product-before.rs').write_bytes(original)
patch=Path('/private/tmp/jarde-return-latch-real-boundary-diagnostic-luna-v1/diagnostic.patch').read_text();assert patch.count('let mut proof_budget = Budget::new')==1;patch=patch.replace('let mut proof_budget = Budget::new(jarde_reader::budget::Limits {\n+                    elapsed_millis:', 'let mut proof_budget = Budget::new(jarde_reader::budget::Limits {\n+                    analysis_steps: u64::MAX,\n+                    elapsed_millis:')
# Keep the reviewed hunk exact after the one added diagnostic budget field.
patch=patch.replace('@@ -13968,4 +13968,236 @@','@@ -13968,4 +13968,237 @@');pp=OUT/'diagnostic-root-v1.patch';pp.write_text(patch)
spec=importlib.util.spec_from_file_location('guard',TEMPLATE);guard=importlib.util.module_from_spec(spec);spec.loader.exec_module(guard);guard.OUT=OUT;guard.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
guard.expected_test_summaries=lambda i,raw:{'actual':[tuple(map(int,x)) for x in guard.SUMMARY_RE.findall(raw.decode())],'expected':[(1,0,0)],'ok':[tuple(map(int,x)) for x in guard.SUMMARY_RE.findall(raw.decode())]==[(1,0,0)]}
env=os.environ.copy()
for k in guard.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(guard.ENV_VALUES);jdk=guard.configure_jdk23(env)
meta=json.loads((ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results/candidate-cli-v1.json').read_text());paths={p for group in ('candidate_sources','test_sources','canonical_files') for p in meta[group]};pins={p:sha(ROOT/p) for p in paths}
record={'schema':'return-latch-real-boundary-diagnostic-root-v1','runner_sha256':sha(Path(__file__)),'patch_sha256':sha(pp),'guard_sha256':sha(TEMPLATE),'pins_before':pins,'commands':[],'jdk':jdk};ok=False
try:
 subprocess.run(['git','apply','--check',str(pp)],cwd=ROOT,check=True);subprocess.run(['git','apply',str(pp)],cwd=ROOT,check=True);subprocess.run(['rustfmt','--edition','2024',str(region)],check=True)
 (OUT/'region-diagnostic.rs').write_bytes(region.read_bytes())
 row=guard.run_command(2,['cargo','test','-p','jarde-java','--lib','--locked','region::tests::diagnose_real_return_latch_boundary_classes','--','--exact','--nocapture'],env);record['commands'].append(row);ok=row['exit_code']==0 and row['guard_stop'] is None and row['test_summary_check']['ok']
finally:
 region.write_bytes(original);record['pins_after_restore']={p:sha(ROOT/p) for p in paths};record['all_52_restored']=record['pins_after_restore']==pins;record['status']='passed' if ok and record['all_52_restored'] else 'failed';(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print({'status':record['status'],'all_52_restored':record['all_52_restored']})
raise SystemExit(0 if record['status']=='passed' else 1)
