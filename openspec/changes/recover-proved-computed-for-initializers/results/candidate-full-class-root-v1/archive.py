from pathlib import Path
import shutil,json,hashlib
ROOT=Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-computed-for-initializers/results')
out=ROOT/'candidate-full-class-root-v1';out.mkdir()
items={
'cli-build':'/private/tmp/jarde-computed-for-cli-build-root-v1',
'call-order':'/private/tmp/jarde-computed-for-call-order-candidate-root-v1',
'controls-direct':'/private/tmp/jarde-computed-for-candidate-comparison-root-v1',
'cf07':'/private/tmp/jarde-computed-for-cf07-final-jdk-replay-root-v1',
'boundary-tests':'/private/tmp/jarde-computed-for-boundary-reconciliation-root-v1',
'preserved-workspace-failure-v1':'/private/tmp/jarde-computed-for-workspace-root-v1'}
extra=[
'/private/tmp/jarde-final-cli-build-root-v1/build-final-cli.py',
'/private/tmp/jarde-computed-for-call-order-candidate-root-runner-v1.py',
'/private/tmp/jarde-computed-for-candidate-verifier-luna-v1/verify.py',
'/private/tmp/jarde-computed-for-call-order-candidate-acceptance-root-v1.json',
'/private/tmp/jarde-computed-for-candidate-comparison-luna-v2/run.py',
'/private/tmp/jarde-computed-for-candidate-comparison-root-verifier-v4.py',
'/private/tmp/jarde-computed-for-candidate-comparison-acceptance-root-v4.json',
'/private/tmp/jarde-computed-for-cf07-final-jdk-replay-luna-v2/replay.py',
'/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v2.py',
'/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v3.py',
'/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v4.py',
'/private/tmp/jarde-computed-for-cf07-final-replay-acceptance-root-v4.json',
'/private/tmp/jarde-computed-for-boundary-reconciliation-root-runner-v1.py',
'/private/tmp/jarde-computed-for-boundary-reconciliation-acceptance-root-v1.json',
'/private/tmp/jarde-compound-workspace-batches-root-v1.py']
for stem in ('call-order-candidate-acceptance-root-v1','candidate-comparison-acceptance-root-v4','cf07-final-replay-acceptance-root-v2','cf07-final-replay-acceptance-root-v3','cf07-final-replay-acceptance-root-v4'):
 for stream in ('stdout','stderr'): extra.append('/private/tmp/jarde-computed-for-'+stem+'.'+stream+'.raw')
records=[]
def cp(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
 a=src.read_bytes();assert a==dst.read_bytes();records.append({'source':str(src),'archived':str(dst.relative_to(out)),'bytes':len(a),'sha256':hashlib.sha256(a).hexdigest()})
for label,source in items.items():
 src=Path(source)
 for p in sorted(src.rglob('*')):
  if p.is_file() and '__pycache__' not in p.parts and 'patch-update' not in str(p):cp(p,out/label/p.relative_to(src))
for src in extra:
 p=Path(src)
 if p.exists():cp(p,out/'executed-programs-and-acceptance'/p.name)
cp(Path(__file__),out/'archive.py')
(out/'archive-inventory.json').write_text(json.dumps(records,indent=2)+'\n')
(out/'README.md').write_text('''# Computed-for 候选完整对照\n\nroot 实际运行并独立接受：call-order 36 命令/60 行、controls-direct 66 命令、CF07 24 命令/80 行。冻结 CLI SHA d1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1、0555、60821472 bytes；4698 个完整源码/包含输入绑定 build+clean。CLI 二进制保留在 receipt 的私有冻结位置，不复制60MB二进制。\n\nordinary f/store/g 保持 while；pending f/g/store 先保存 f 再调用 g，随后安全 for，正常/异常观察全同原。direct-call limit() 初值为 for 且一次调用。CF07 default/all 整类原样双JDK重编运行同原，全部 lastIndexOf/count​ed 物理来源、完整 for/if跨度与单一loop所有者通过。JADX源来自已接受冻结基线，没有新的去编译声明。已知普通controls的JADX四组调用重排错误单列；八Jarde controls整类编译失败/no runtime，完整正文/quality/map/拒绝边界同旧。\n\nboundary-tests 实际 core1/0、五suite118/0，含尾部嵌套/multiblock shape-only拒绝、真实错误目标/额外返回入口与公共Stop原子输出；不冒称这些shape扰动是可验证class。\n\npreserved-workspace-failure-v1 是真实第84批旧while期待失败，不作为全workspace通过。CF07 checker v2/v3 两次harness断言失败raw保留，root v4 修正默认regions为空及历史Runner目录期待，精确oracle8行、隔离classpath和物理span约束均保留。\n\n复制保留原receipt和raw字节/原绝对路径。archive-inventory绑定每份复制。执行时checker核live4698源码；此后永久回归变化须fresh完整workspace和最终CLI closure构建，不能把本目录当自身CI或最终clean交付。\n'''.replace('count​ed','counted'))
print({'copied':len(records),'bytes':sum(x['bytes'] for x in records)})
