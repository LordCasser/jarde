from pathlib import Path
import json,hashlib,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde')
S=Path('/private/tmp/jarde-nested-loops2-same-ir-root-v1')
O=R/'openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops2-same-ir-root-v1'
e=json.loads((S/'execution.json').read_text())
assert e['status']=='passed_same_real_IR_diagnostic_only' and e['restored'] and e['target_absent']
assert e['normalization_production_exact'] and e['source_pins_before']==e['source_pins_after_restore']
assert e['commands'][0]['exit_code']==e['clean_command']['exit_code']==0 and e['commands'][0]['guard_stop'] is None
assert json.loads(Path('/private/tmp/jarde-cf11-invocations-root-v1/nested-loops2-same-ir/execution.json').read_text())['exit_code']==0
raw=(S/'0.stderr.raw').read_bytes()
assert b'at_bci=4 for_header_reject gate=unsupported_test_operation bci=6' in raw
assert b'at_bci=26 for_header_reject gate=region_rs_line_13597' in raw
for row in [*e['commands'],e['clean_command']]:
 for stream in row['streams'].values():
  p=Path(stream['path']);assert p.stat().st_size==stream['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==stream['sha256']
O.mkdir(parents=True,exist_ok=False);rows=[]
def cp(src,rel):
 dst=O/rel;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
 raw=src.read_bytes();assert dst.read_bytes()==raw
 rows.append({'source':str(src),'archived':str(rel),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
for label,base in {'execution':S,'invocation':Path('/private/tmp/jarde-cf11-invocations-root-v1/nested-loops2-same-ir')}.items():
 for p in sorted(base.rglob('*')):
  if p.is_file():cp(p,Path(label)/p.relative_to(base))
for version in range(1,5):
 base=Path(f'/private/tmp/jarde-nested-loops2-diagnostic-prep-luna-v{version}')
 for name in ('pins.json','README.md','region.rs.patch'):
  cp(base/name,Path('private-prep')/f'v{version}'/name)
cp(Path('/private/tmp/jarde-nested-loops2-diagnostic-prep-luna-v4/region.rs'),Path('private-prep/v4/region.rs'))
cp(Path('/private/tmp/jarde-nested-loops2-diagnostic-root-v1.py'),Path('runner/jarde-nested-loops2-diagnostic-root-v1.py'))
cp(Path(__file__),Path('archive-root-v1.py'))
(O/'archive-inventory.json').write_text(json.dumps({'files':rows},indent=2)+'\n')
(O/'README.md').write_text('''# CF11 TestNestedLoops2：同次 IR 的实际 ForHeader 首拒

Root在593fbcc0b主线实际运行一个临时库test，原878-byte javac23 class经Reader/canonical/SSA/NormalFlowView/recover。同次10节点/12边/28SSA指令/19trace；完整4707输入恢复，默认栈、5GiB free/target1GiB一秒守卫未触发，target peak156280445bytes、clean0/target不存在。生产源码去除logger/test后逐字节恒同。实际调用与stdout/stderr在invocation；source pins、class/javap绑定和原raw见execution。

外层header4：唯一latch41，init3写slot3，update41为iinc、transfer44，准确两输入phi及前置门通过；随后在条件Invoke List.size@6明确拒绝，返回Ok(None)。此前静态候选由本次真实trace证实。内层header26在当前preheader末尾不是j初始化store而拒绝，不应强制成for。最终恢复仍structured且无fallback，外/内while，十节点均claimed。

private v1/v2/v3未运行：root审查先修preheader引用类型、真实Boolean返回标记、两个Option格式。实际运行v4；原稿patch/pins保留。测试调用Some(false),true与原方法flags/descriptor匹配；不借错误false的旧诊断。没有实现生产白名单扩展或接受任何新for。

下一方案保持现有init/update/phi/invariant/owner判据，复用header_tested_loop已有test_is_pure与Builder条件恢复，让已证条件中的非local-writing producer可留在for头同一求值位置；新增effect/throw控制后再按OpenSpec实施。泛型raw List投影独立记录，不混入该改动。整个CF11与71/612计数不变。
''')
print(json.dumps({'output':str(O),'files':len(rows),'execution_sha256':hashlib.sha256((S/'execution.json').read_bytes()).hexdigest()}))
