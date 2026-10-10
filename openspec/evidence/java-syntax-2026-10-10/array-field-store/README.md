# Array field store baseline

This is a single-class Java 8 probe for the EM-18 `TestArrayInit.test2` family: an ordered fresh
`byte[]` initializer whose final consumer is an instance-field `putfield`. `storeLiteral` keeps the
original instance-method shape. Static `replace` makes the nullable field target explicit so the
Runner can observe whether initializer effects run before the final field-store null check.

The Runner observes a successful replacement, a second-element exception that must leave the old
field value intact, a null target whose elements complete, and a null target whose second element
throws. The values below are review expectations only; they are not fixture truths. The first two
fresh Java 8 original runs determine the raw stdout, stderr, and exit oracle independently per JDK.

```text
success=[11, 22, 33]:fresh=true:trace=123
failure=IllegalStateException:element-2:same=true:values=[10, 20, 30]:trace=12
null=NullPointerException:trace=123
null-failure=IllegalStateException:element-2:trace=12
```

Run [prepare-baseline-luna-v1.py](prepare-baseline-luna-v1.py) only after root review. It writes a
fresh `baseline-root-v1` directory, recompiles the complete class and Runner with each frozen JDK,
freshly decompiles both class inputs with JADX default and `--rename-flags none`, and renders both
with the frozen Jarde CLI. Every candidate source set is compiled with empty classpath/sourcepath,
then run with `-Xverify:all` using only its new classes. The script records command argv, exits, raw
streams, full sources/documents, generated class files, and hashes. Root has run and independently
verified this script; the execution record follows.

# Root 实际验收

2026-10-10 root 已运行脚本。原程序双真实 JDK 2/2、fresh JADX default/none 四腿 4/4、当前冻结 Jarde CLI 两腿 2/2，31 条真实命令全部成功。所有完整生成源码以空 CP/SP 重编，仅新 classes `-Xverify:all` 运行；原程序四条 raw 轨迹逐字一致。成功替换 fresh=true，第二元素抛错 same=true 且旧值保留，null target 先完整求值 trace123，null 上第二元素抛错为 IllegalStateException/trace12。

root 独立 verifier 实际 294 checks/0 errors，核对封闭文件 inventory、原始流、全部七方法/两字段的物理名称/descriptor/access flags、两个 store 方法的 allocation/元素/调用/putfield/return BCI 来源。CLI SHA `71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110`，结果见 `root-verification-v1.json`。本片证明现有 ArrayInitializers 与 FieldWrite 消费点组合有效，不新增实现 change，也不把这一场景计作整个 EM-18 已追平。静态/构造器初始化位置另行分析。
