from pathlib import Path
import hashlib,json,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=R/'openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops4-same-ir-root-v1'
BASE=Path('/private/tmp/jarde-cf11-testnestedloops4-diagnostic-root-v2')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
x=json.loads((BASE/'execution.json').read_text())
assert x['status']=='passed_same_real_IR_diagnostic_only' and x['restored'] and x['target_absent']
assert x['source_pins_before']==x['source_pins_after_restore'] and len(x['source_pins_before'])==4700
assert x['runner_sha256_before']==x['runner_sha256_after']==H('/private/tmp/jarde-cf11-testnestedloops4-diagnostic-root-v2.py')
for name,sha in x['source_pins_before'].items():assert H(R/name)==sha
for c in x['commands']+[x['clean_command']]:
 assert c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3
 for rec in c['streams'].values():assert H(rec['path'])==rec['sha256'] and Path(rec['path']).stat().st_size==rec['bytes']
assert b'test result: ok. 1 passed; 0 failed; 0 ignored;' in (BASE/'0.stdout.raw').read_bytes()
stderr=(BASE/'0.stderr.raw').read_text()
assert stderr.count('CF11_INSN ')==40 and stderr.count('CF11_EDGE ')==19 and stderr.count('CF11_NODE ')==15
assert 'at_bci=28 region_return' in stderr and 'refusal one_arm_continuation branch_bci=12 join=Some(62)' in stderr
assert not (R/'target').exists()
O.mkdir();rows=[]
def cp(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst);assert dst.read_bytes()==src.read_bytes()
 rows.append({'source':str(src),'archived':str(dst.relative_to(O)),'bytes':src.stat().st_size,'sha256':H(src)})
for label in ['v1','v2']:
 p=Path('/private/tmp/jarde-cf11-testnestedloops4-diagnostic-root-'+label)
 for f in sorted(p.rglob('*')):
  if f.is_file():cp(f,O/label/f.relative_to(p))
 for suffix in ['.py','.stdout','.stderr']:
  src=Path(str(p)+suffix);cp(src,O/'runner'/src.name)
for version in ['v2','v3']:
 d=Path('/private/tmp/jarde-cf11-testnestedloops4-diagnostic-luna-'+version)
 for name in ['region.rs.patch','pins.json','review.md','run-command.txt']:cp(d/name,O/'private-diagnostic'/version/name)
cp(Path('/private/tmp/jarde-computed-for-closure-clean-audit-root-v1/execution.json'),O/'clean-main-before-diagnostic.json')
cp(Path(__file__),O/'archive-and-check.py')
(O/'archive-inventory.json').write_text(json.dumps(rows,indent=2)+'\n')
(O/'README.md').write_text('''# CF11 同一次真实 IR：退出路径与 arm 接续

产品基线8bef1b76，证据主线992c3ca0；未修改生产算法。Root实际运行一个精确库诊断测试通过，同一个602-byte原class经过Reader/canonical/SSA/NormalFlowView/recover。日志包含15节点/19normal边、40条物理SSA指令、完整SSA Debug、dom/ipdom、自然环及23条递归/候选trace。4700源码输入完全恢复，root runner前后自pin一致；peak157430473 bytes，cargo clean exit0、target不存在。原v1三logger Display格式编译错误101以及clean记录完整保留；v2只修为Debug，非产品修复。

真实观察：outer natural loop header5；middle header17自然环仅17/23/56，nested header31自然环31/39/47。outer Frame允许内层路径，middle body Frame scope却不包含node28（global index8）及内层环；region_at28返回空Straight/None，未进入31。middle返回临时Loop17/If23(joinNone)，body缺内层loop/return/break。最终branch10的单arm接续在join62拒绝，整个公开结果保持fallback。Frame数字是NormalFlowView索引；arm boundary Some(5)对应BCI62，并非BCI5，不能误诊边界转换。

结合实际Region形状与当前源码4635附近可推导：loop_arm_join_source的closed_body不接受If(joinNone)，首先拒绝这一候选；这是源码推导，不冒称该helper内部另有实测trace。后继owner_nodes==natural_loop和唯一正常exit判据也不能直接承载完整nested break/early-return正文。仅放宽外层join/closed_body会冒险发布漏掉内层的结构，因此禁止单独修拒绝门。

下一步需确定如何用已有Frame/Region/transfer/return证书，证明从middle自然环23→28进入、包含nested31、到middle退出62或准确return45的完整退出路径所有权；自然环集合不能扩大为“可到达即正文”。必须核全部canonical入/出边、path、上层scope、唯一入口、重复所有者、额外正常/异常边、预算/取消与失败回滚。再设计arm接续消费该完整证明；不强制三for、不新增未经论证的通用walker/pass/IR。这是诊断证据与方案约束，尚未创建实现OpenSpec或接受实现；CF11/71未关闭。
''')
print(json.dumps({'status':'accepted_diagnostic_capture_and_restore','files':len(rows),'bytes':sum(r['bytes'] for r in rows)}))
