import hashlib,importlib.util,json,shutil,datetime
from pathlib import Path
R=Path('/Users/lordcasser/workspace/projects/jarde');P=Path('/private/tmp/jarde-conditional-constructor-values-baseline-root-v1');O=R/'openspec/changes/recover-proved-constructor-conditional-values/results/control-baseline-root-v1'
assert not O.exists();H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
x=json.loads((P/'execution.json').read_text());v=json.loads((P/'checker-root-v6.json').read_text())
assert H(P/'execution.json')=='ea9563353916d1b690883328fa0a304a0997fdaf29aaba8f61417fc73767e7e2'
assert x['status']=='observed' and x['actual_command_count']==39 and len(x['commands'])==39 and x['source_tool_inputs_unchanged']
assert v['status']=='verified_observation_capture_with_quality_gaps' and v['actual_command_count']==39 and v['raw_stream_count']==78 and not v['behavior_mismatches']
oracle=None;runs=[];failures=[]
for c in x['commands']:
 assert c['guard_stop'] is None
 for stream in ['stdout','stderr']:
  z=c['streams'][stream];q=Path(z['path']);assert q.stat().st_size==z['bytes'] and H(q)==z['sha256']
 if c['label'].endswith('-run'):
  b=Path(c['streams']['stdout']['path']).read_bytes();assert c['exit_code']==0 and not Path(c['streams']['stderr']['path']).read_bytes()
  if oracle is None:oracle=b
  assert b==oracle;runs.append(c['label'])
 if '-jarde-' in c['label'] and c['label'].endswith('-compile'):
  assert c['exit_code']==1;failures.append(c['label'])
assert len(runs)==6 and len(oracle.splitlines())==19 and len(failures)==12
assert hashlib.sha256(oracle).hexdigest()=='a06d6cc59147b779796efa914406f8ff35028263a534906893de5ee2dabf5c7e'
launch=json.loads(Path('/private/tmp/jarde-conditional-constructor-values-baseline-launch-receipt-root-v1.json').read_text());build=json.loads(Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3/execution.json').read_text())
assert launch['collector_exit_code']==0 and launch['checker_exit_code']==1 and launch['source_pins_before']==launch['source_pins_after']==build['source_pins_after']
p=Path('/private/tmp/jarde-for-condition-next-round-luna-v1/jarde-for-condition-ci-verifier-luna-v4.py');assert H(p)=='cc82f638e0292196020bee8422e62dc033c371565a7afc38c56d2b971e0df673'
s=importlib.util.spec_from_file_location('closure',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);live=m.pin_sources(m.source_paths());assert len(live)==4718 and live==build['source_pins_after'] and not (R/'target').exists()
O.mkdir(parents=True);shutil.copytree(P,O/'actual');shutil.copytree('/private/tmp/jarde-conditional-constructor-values-inputs-root-v1',O/'frozen-inputs')
shutil.copytree('/private/tmp/jarde-for-condition-checker-root-v6',O/'checker-v6')
for name in ['conditional-constructor-baseline-root-v1','conditional-constructor-baseline-checker-root-v6']:
 shutil.copytree(Path('/private/tmp/jarde-cf11-invocations-root-v1')/name,O/'invocations'/name)
shutil.copyfile('/private/tmp/jarde-conditional-constructor-values-baseline-launch-root-v1.py',O/'launch-root-v1.py');shutil.copyfile('/private/tmp/jarde-conditional-constructor-values-baseline-launch-receipt-root-v1.json',O/'launch-receipt-root-v1.json');shutil.copyfile(__file__,O/'accept-root-v1.py')
files=[{'path':str(q.relative_to(O)),'bytes':q.stat().st_size,'sha256':H(q)} for q in sorted(O.rglob('*')) if q.is_file()]
for row in files:assert H(O/row['path'])==row['sha256']
a={'schema':'conditional-constructor-control-baseline-acceptance-root-v1','status':'accepted_old_cli_observations_only','accepted_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'execution_sha256':H(P/'execution.json'),'cli':x['cli'],'command_count':39,'raw_stream_count':78,'oracle_runtime_legs':runs,'oracle_lines':19,'oracle_sha256':hashlib.sha256(oracle).hexdigest(),'jarde_compile_failures':failures,'jarde_runtime_legs':0,'source_closure_count':len(live),'source_closure_equal_accepted_final_build':True,'target_absent':True,'checker_v5_failure_preserved':True,'checker_v6_actual_exit':0,'conditional_product_recovery_accepted':False,'archive_inventory':files}
(O/'root-acceptance.json').write_text(json.dumps(a,indent=2)+'\n')
(O/'README.md').write_text('''# 构造条件值旧 CLI 控制基线（实际观察）\n\nRoot 接受 39 命令/78 raw，双 JDK 原类及 JADX default/none 四组完整类共六 runtime 腿的 19 行输出完全相同，含对象身份、调用顺序、null 构造失败、字段赋值失败与 catch。Jarde default/all 共 12 次 combined/isolated 编译失败，没有 Jarde runtime；隔离仍使用相同生成源码，不算独立输入。不编辑生成 Java。\n\n旧 CLI 为上一已交付产品的 final-cli-root-v3，绑定 final-lint-build-root-v3 的 4718 完整输入闭包。本次不是新实现验收。Collector 成功，v5 checker 因无 handler 假设实际失败，保留原 launcher failure/原 raw；v6 只补物理 Code 异常表与 javap 有序完整表、BCI 边界核验，独立执行 exit0。v6 仍明确 quality gaps/behavioral acceptance not claimed，root 仅接受旧观察及原/JADX oracle。\n\n所有文件见 root-acceptance.json 的逐项归档 hash。无 Rust 构建，target 一直不存在。\n''')
t=R/'openspec/changes/recover-proved-constructor-conditional-values/tasks.md';text=t.read_text();assert '- [ ] 1.2 ' in text;t.write_text(text.replace('- [ ] 1.2 ','- [x] 1.2 ',1))
print(json.dumps({k:a[k] for k in ['status','command_count','raw_stream_count','oracle_lines','jarde_runtime_legs','source_closure_count','target_absent']}))
