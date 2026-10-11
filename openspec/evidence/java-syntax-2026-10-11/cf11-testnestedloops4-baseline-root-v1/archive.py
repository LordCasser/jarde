from pathlib import Path
import hashlib,json,shutil
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops4-baseline-root-v1'
BASE=Path('/private/tmp/jarde-cf11-testnestedloops4-baseline-root-v1')
A=Path('/private/tmp/jarde-cf11-testnestedloops4-acceptance-root-v1.json')
a=json.loads(A.read_text());assert a['status']=='accepted_scoped_gap_observation'
assert a['execution']['sha256']==hashlib.sha256((BASE/'execution.json').read_bytes()).hexdigest()
OUT.mkdir();rows=[]
def cp(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst);raw=src.read_bytes();assert dst.read_bytes()==raw
 rows.append({'source':str(src),'archived':str(dst.relative_to(OUT)),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
for p in sorted(BASE.rglob('*')):
 if p.is_file():cp(p,OUT/'capture'/p.relative_to(BASE))
for name in ['jarde-cf11-testnestedloops4-collector-root-v1.py','jarde-cf11-testnestedloops4-verifier-root-v1.py','jarde-cf11-testnestedloops4-acceptance-root-v1.json','jarde-cf11-testnestedloops4-acceptance-root-v1.stdout','jarde-cf11-testnestedloops4-acceptance-root-v1.stderr','jarde-computed-for-cf07-final-replay-verifier-root-v4.py']:
 cp(Path('/private/tmp')/name,OUT/'verification'/name)
cp(Path(__file__),OUT/'archive.py')
(OUT/'archive-inventory.json').write_text(json.dumps(rows,indent=2)+'\n')
(OUT/'README.md').write_text('''# CF11：固定 TestNestedLoops4 完整类差异

本次观察绑定产品 `8bef1b76acfc93e1b1510e8cb71d7c7a25b0fdc6` 的已构建冻结 CLI，未修改生产代码。上游参考 checkout `2fb1b16386941660fda07e9017285aec40fcb37f`；TestNestedLoops4.java SHA b84dbf9988e1343f6d26f06345a4e8ba8b17993b4dc33419c86d9e9194968cca。移除测试框架与嵌套 owner 外壳，将准确 testFor 方法字节切片放入 cf11.UpstreamNestedLoops4；明确是适配输入，不是上游整个 nested class 的字节等同。

Root 实际执行25命令，并冻结独立 checker 实际 exit0；83个捕获文件、完整源码、50 raw streams、class/JAR 和工具输入均保留。JDK8/23 两个原类执行均输出 testFor=12；现有 JADX 1.5.6 新鲜运行 default/rename-none 两profile，同一个 javac23 单class JAR，输出完整源码在两个 JDK 的四腿编译执行全部同原。上游参考源码与本机 JADX runtime 版本分别记录，没有声称构建参考 checkout。

四份 Jarde 完整源码 default/all/JDK8/23 均渲染成功、编译 exit1（缺少返回语句），没有运行或生成 class。testFor 为 Fallback/ExplanationOnly，准确 jre_region_arms_do_not_meet / block10；default/all 全文和来源一致。独立 classfile 与 javap 核准 constructor3/testFor40条物理BCI、方法序号/flags/Code envelope、UTF8来源和精确 owner/name/descriptor；未独立计算报告里的 BLAKE3 digest。所有输入与collector前后pins恒同，守卫5GiB free/target1GiB，无cargo构建，target峰值0；env_overrides字段是复用守卫模板的元数据。

固定方法返回12，但 return0在该常量输入中动态不可达，不能声称该运行覆盖了早返回。下一步需同一次真实IR确认 normal graph/dom/ipdom、三层loop scope以及branch10的候选join和首个失败路径。BCI62仅是物理汇合候选，不当作已获证明的join；不得凭“arms不汇合”文本直接推断修复。outer复合latch可正确呈现while，验收不强制三个for。当前只接受缺口观察，不接受实现，不关闭整CF11/71单元。
''')
print(json.dumps({'files':len(rows),'bytes':sum(r['bytes'] for r in rows)}))
