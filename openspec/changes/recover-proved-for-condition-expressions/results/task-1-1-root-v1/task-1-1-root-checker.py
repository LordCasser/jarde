from pathlib import Path
import json,hashlib,re
R=Path('/Users/lordcasser/workspace/projects/jarde')
B=R/'openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops2-baseline-root-v1'
I=R/'openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops2-same-ir-root-v1'
H=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for directory in (B,I):
 manifest=json.loads((directory/'archive-inventory.json').read_text())
 for row in manifest['files']:
  p=directory/row['archived'];assert p.stat().st_size==row['bytes'] and H(p)==row['sha256']
check=json.loads((B/'observation/checker-root-v1.json').read_text())
assert check['status']=='verified_observation_capture' and check['command_count']==29
ir=json.loads((I/'execution/execution.json').read_text())
assert ir['status']=='passed_same_real_IR_diagnostic_only' and ir['restored'] and ir['target_absent'] and ir['normalization_production_exact']
assert ir['source_pins_before']==ir['source_pins_after_restore'] and len(ir['source_pins_before'])==4707
assert all(H(R/p)==v for p,v in ir['source_pins_after_restore'].items())
raw=(I/'execution/0.stderr.raw').read_text()
assert ir['marker_counts']=={'CF11_LOOPS2_CLASS':1,'CF11_LOOPS2_EDGE':12,'CF11_LOOPS2_NODE':10,'CF11_LOOPS2_SSA_TABLE':1,'CF11_LOOPS2_INSN':28,'CF11_LOOPS2_TRACE':19,'CF11_LOOPS2_RECOVERY':1}
assert 'at_bci=4 for_header_reject gate=unsupported_test_operation bci=6' in raw
assert 'at_bci=26 for_header_reject gate=region_rs_line_13597' in raw
v22=raw.split('        SsaValue {')[23]
assert 'bci: 6,' in v22[:230] and v22.count('SsaUse {')==1
use=v22[v22.index('            uses:'):v22.index('            replaced_by:')]
assert 'bci: 4,' in use and '11,' in use
report=Path('/private/tmp/jarde-for-condition-design-review-luna-v1.md').read_text()
report=report.replace('外层 init(4)、update(3)、phi 门通过','外层 header4、init3、update41、phi 门通过')
report+='\nRoot复核：v1正文开头init/update数字误写已在此副本纠正；本次IR实际ValueId22定义在block4/BCI6，SSA值表仅一个use为同block branch@11，与instruction输出一致。根验收不依赖v1的典型字节码推测。\n'
O=R/'openspec/changes/recover-proved-for-condition-expressions/results/task-1-1-root-v1';O.mkdir(parents=True,exist_ok=False)
(O/'design-review-root-corrected.md').write_text(report)
(O/'acceptance.json').write_text(json.dumps({'status':'accepted_fixed_baseline_same_ir_only','baseline_command_count':29,'same_ir_counts':ir['marker_counts'],'outer_header':4,'init_bci':3,'update_bci':41,'first_reject':{'operation':'Invoke List.size','bci':6},'invoke_result':{'id':22,'definition_block':4,'definition_bci':6,'uses':[{'block':4,'bci':11}]},'inner_reject':'preheader last instruction is not induction store','same_ir_execution_sha256':H(I/'execution/execution.json'),'source_restored_count':4707,'target_absent':not(R/'target').exists()},indent=2)+'\n')
(O/'task-1-1-root-checker.py').write_bytes(Path(__file__).read_bytes())
p=R/'openspec/changes/recover-proved-for-condition-expressions/tasks.md';s=p.read_text();assert '- [ ] 1.1' in s;p.write_text(s.replace('- [ ] 1.1','- [x] 1.1',1))
print('Accepted task1.1 fixed whole-class baseline + actual same-IR gate; source restored, no implementation applied')
