# EM-12：`super` 与词法外层 `Outer.super` 的源码绑定

固定基准为 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`。可重放的 [replay.py](replay.py) 校验清单中四项测试及 `ConstructorVisitor`、`InlineMethods`、`InsnGen` 的 SHA-256；[baseline/summary.json](baseline/summary.json) 记录输入、原 class 与 Jarde CLI 的哈希。本次锁定 `TestSuperInvoke` 的普通父类方法、`TestSyntheticInline2` 的词法外层父类方法两个语义方向。`TestSuperInvokeWithGenerics` 是泛型父类构造重载的源码文本断言，`TestShadowingSuperMember` 主要是父类同名**字段**绑定，分别属于 EM-05/EM-07 交叉边界；`TestSyntheticInline2.test()` 还显式 `disableCompilation()`，不能把其全部文本断言算成 Java 8 重编验收。

正向冻结样本复用已存在的 `OuterReceiverCases`。原始、固定 JADX 和当前 Jarde 的完整依赖源码均经 `javac --release 8` 重编、`java -Xverify:all` 运行，逐字输出 `20:10:1:3`。四段结果分别区分显式 `other.state`、捕获的词法 `Outer.this.state`、父类 `value()` 与成员自身 `value()`。Jarde 的家族报告为 `projected`，生成 `OuterReceiverCases.super.value()`，并以物理来源记录被隐藏的 synthetic 桥。因此 `tests/member_family_identity.rs::outer_super_method_bridge_is_not_projected` 在基线就失败的原因是**断言过时**；这条正向投影不应被回退为拒绝。不过样本中未使用的 `StaticNested` 仍未进入 Jarde 根源码，不能借该运行声称所有嵌套成员家族已完整追平。

反例 [input/em12](input/em12) 的 `Parent` 同时声明 `pick(Arg)` 与 `pick(NarrowArg)`，且所选完整类层级有 `NarrowArg extends Arg`。成员方法的形参静态类型为 `Arg`，字节码在 BCI 2 通过 Outer 的 synthetic 桥准确 `invokespecial Parent.pick(Arg)`；`Arg` 静态值不能隐式向下转换为 `NarrowArg`，所以 `Outer.super.pick(value)` 的 Java 8 源级绑定仍应是 `pick(Arg)`。原 class 与 JADX 的**全部五个类**源码均重编、验证运行输出 `number`。Jarde 记录 `Outer.super source binding refused: source hierarchy contains a competing same-name method`，未投影成员，保留无法恢复方法体的桥，完整源码在 `Case.java:41` 缺返回而编译失败。这是当前首片的真实缺口，不是 JVM 分派疑问；编译日志、两侧源码、物理 `javap` 与哈希均在 [baseline/binding](baseline/binding)。

架构上不用新通用机制。现有桥体准确目标、捕获接收者、实参/异常顺序、可见调用闭包及家族 writer 均已存在；过宽的是最后 `prove_outer_super_source_binding` 的“任一同名方法即拒绝”。[窄 OpenSpec](../../changes/prove-outer-super-overload-binding/proposal.md)要求再证明**生成源码实参的静态类型**和竞争者不适用性，仅对同轮已选完整 class 层级放行。不能仅凭桥 descriptor 推断源码静态类型，也不能照搬 JADX 的文本 cast 而跳过目标绑定校验。泛型、varargs、装箱、checked exception、无法解析的类层级以及同型显式 `other` 仍应保守拒绝。

重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/em12-super-dispatch/replay.py \
  --jarde /tmp/jarde-cli-accepted-em06 \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/em12-audit-replay
```

`--out` 必须是空目录。当前脚本把第二片 Jarde 编译失败作为冻结预期；实施后应改为第三侧完整重编和 `number` 运行的验收。编译中间物、JAR 与临时类自动删除；这里只保存可审查的文本证据。
