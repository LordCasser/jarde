from pathlib import Path
import shutil,json,hashlib
root=Path('/Users/lordcasser/workspace/projects/jarde');out=root/'openspec/changes/recover-proved-computed-for-initializers/results/final-local-product-validation-root-v1';out.mkdir();rows=[]
def cp(p,d):
 d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);a=p.read_bytes();assert a==d.read_bytes();rows.append({'source':str(p),'archived':str(d.relative_to(out)),'bytes':len(a),'sha256':hashlib.sha256(a).hexdigest()})
for label,src in [('workspace','/private/tmp/jarde-computed-for-workspace-root-v2'),('lint','/private/tmp/jarde-computed-for-final-lint-root-v1'),('final-cli-build','/private/tmp/jarde-computed-for-final-cli-build-root-v2'),('strict','/private/tmp/jarde-computed-for-strict-root-v1')]:
 base=Path(src)
 for p in sorted(base.rglob('*')):
  if p.is_file():cp(p,out/label/p.relative_to(base))
names=['jarde-computed-for-workspace-acceptance-root-v2.json','jarde-computed-for-final-lint-root-runner-v1.py','jarde-computed-for-final-lint-verifier-root-v1.py','jarde-computed-for-final-lint-acceptance-root-v1.json','jarde-computed-for-final-cli-verifier-root-v1.py','jarde-computed-for-final-cli-acceptance-root-v1.json','jarde-compound-workspace-batches-root-v1.py']
for stem in ['jarde-computed-for-workspace-acceptance-root-v2','jarde-computed-for-final-lint-acceptance-root-v1','jarde-computed-for-final-cli-acceptance-root-v1']:
 for stream in ['stdout','stderr']:names.append(stem+'.'+stream+'.raw')
for name in names:cp(Path('/private/tmp')/name,out/'executed-programs-and-acceptance'/name)
cp(root/'openspec/changes/project-integer-constant-names-with-refused-member-family/results/full-workspace-final-cli-root-v1/verify-workspace-v2.py',out/'executed-programs-and-acceptance/verify-workspace-v2.py')
cp(Path('/private/tmp/jarde-final-cli-build-root-v1/build-final-cli.py'),out/'executed-programs-and-acceptance/build-final-cli.py')
cp(Path(__file__),out/'archive.py')
paths=['.github/workflows/ci.yml','Cargo.toml','Cargo.lock','crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs','crates/jarde-java/tests/p3_loop_exit_gateways.rs','crates/jarde-java/tests/p3_loop_terminal_return.rs','tests/p3_for_latch.rs','tests/fixtures/p3-loop-terminal/LoopCases.class','tests/fixtures/p3-for-latch/v8/ForBoundaries.class','openspec/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/direct-return-actual/javac8/classes/ComputedForDirectReturn.class','openspec/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/actual-controls/javac8/original/ComputedForControls.class','openspec/changes/recover-proved-computed-for-initializers/results/shared-stack-input-root-v1/javac8/original/classes/SharedStoreInput.class']
text='# 产品自身CI源码核对表\n\n源码表仅列关键入口/回归；实际验收还须重建并绑定完整4700输入到产品Git/live/最终CLI。自身CI未执行，不能借此前产品CI。\n\n| Path | SHA-256 |\n| --- | --- |\n'
for p in paths:text+='| '+p+' | '+hashlib.sha256((root/p).read_bytes()).hexdigest()+' |\n'
(out/'ci-source-checklist.md').write_text(text)
(out/'archive-inventory.json').write_text(json.dumps(rows,indent=2)+'\n')
(out/'README.md').write_text('''# 当前产品本地完整验收\n\nroot独立接受fresh workspace v2：95批、359个metadata目标和359份结果，3451 passed/0 failed/97 ignored，8零测试目标按实际artifact核对。4700源码/包含/fixture闭包恒同，峰值811131401bytes、最低free19350835200bytes；已删除完成测试可执行文件并最终cargo clean，无target。\n\n最终fmt与同CI29项Clippy允许表的全workspace/all-targets/all-features/locked -D warnings三命令通过，root独立核CI argv一致并clean。OpenSpec strict349/0。最终CLI build+clean两命令真实通过，0555/60821472bytes/SHA d1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1，完整4700输入同workspace/live/冻结closure。\n\n最终CLI与已执行对照的候选CLI逐字节相同；root verifier绑定四份实际接受receipt（正常对照与共享输入可靠拒绝分别记录），未宣称重新执行146条对照命令。二进制保留receipt私有冻结位置，不复制60MB二进制。\n\n源码产品尚待提交推送及自身CI核验；本目录不冒称Git/CI或clean main已交付。原raw/receipt路径字节保留，inventory绑定复制。\n''')
print({'files':len(rows),'bytes':sum(r['bytes'] for r in rows)})
