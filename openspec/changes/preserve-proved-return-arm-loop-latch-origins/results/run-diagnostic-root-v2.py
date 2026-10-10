from pathlib import Path
import hashlib,importlib.util,json,os,subprocess,sys,traceback
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=Path('/private/tmp/jarde-lastindex-diagnostic-root-v2')
PATCH=Path('/private/tmp/jarde-cf07-lastindexof-region-diagnostic-v1/diagnostic-root-v2.patch')
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
META=ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results/candidate-cli-v1.json'
TEST='region::tests::diagnose_cf07_last_index_latch_region_ownership_from_real_class_ir'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(TEMPLATE)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(PATCH)=='fe18353247b4e744886b3335395253baa13b2d046a635bf39fe8972625c1e1fa'
meta=json.loads(META.read_text()); pins={k:v for group in ['candidate_sources','test_sources','canonical_files'] for k,v in meta[group].items()}
assert len(pins)==52 and all(sha(ROOT/p)==s for p,s in pins.items())
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip()=='1f386686c6a31813254a85fed48d2af0af84a61c'
assert subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)==b''
assert not OUT.exists();OUT.mkdir(); (OUT/'diagnostic.patch').write_bytes(PATCH.read_bytes())
region=ROOT/'crates/jarde-java/src/region.rs'; original=region.read_bytes(); (OUT/'region-before.rs').write_bytes(original)
spec=importlib.util.spec_from_file_location('guard',TEMPLATE);guard=importlib.util.module_from_spec(spec);spec.loader.exec_module(guard);guard.OUT=OUT
# Private diagnostic raw stays outside repo; otherwise reuse the reviewed actual guard.
guard.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def summary(i,raw):
 actual=[tuple(map(int,x)) for x in guard.SUMMARY_RE.findall(raw.decode())]
 return {'actual':actual,'expected':[(1,0,0)],'required_test':TEST,'ok':actual==[(1,0,0)] and f'test {TEST} ... ok' in raw.decode()}
guard.expected_test_summaries=summary
env=os.environ.copy()
for key in guard.STRIPPED_ENV_KEYS:env.pop(key,None)
env.update(guard.ENV_VALUES); jdk=guard.configure_jdk23(env)
rows=[];ok=False;record={'schema':'cf07-lastindex-real-region-diagnostic-root-v1','product_commit':'1f386686c6a31813254a85fed48d2af0af84a61c','pins_before':pins,'patch_sha256':sha(PATCH),'guard_template_sha256':sha(TEMPLATE),'runner_sha256':sha(Path(__file__)),'jdk':jdk,'commands':rows}
try:
 subprocess.run(['git','apply','--check',str(PATCH)],cwd=ROOT,check=True)
 subprocess.run(['git','apply',str(PATCH)],cwd=ROOT,check=True)
 subprocess.run(['rustfmt','--edition','2024',str(region)],cwd=ROOT,check=True)
 (OUT/'region-with-diagnostic.rs').write_bytes(region.read_bytes());record['diagnostic_region_sha256']=sha(region)
 row=guard.run_command(2,['cargo','test','-p','jarde-java','--lib','--locked',TEST,'--','--exact','--nocapture'],env);rows.append(row)
 ok=row['exit_code']==0 and row['guard_stop'] is None and row['test_summary_check']['ok']
except Exception as exc:
 record['error']=f'{type(exc).__name__}: {exc}';traceback.print_exc()
finally:
 region.write_bytes(original)
 record['pins_after_restore']={p:sha(ROOT/p) for p in pins}
 record['all_52_restored']=record['pins_after_restore']==pins
 clean=guard.run_command(3,['cargo','clean'],env);rows.append(clean)
 record['target_absent_after_clean']=not (ROOT/'target').exists()
 ok=ok and record['all_52_restored'] and clean['exit_code']==0 and clean['guard_stop'] is None and record['target_absent_after_clean']
 record['status']='passed' if ok else 'failed'
 (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'commands':len(rows),'all_52_restored':record['all_52_restored'],'target_absent':record['target_absent_after_clean'],'error':record.get('error')}))
raise SystemExit(0 if ok else 1)
