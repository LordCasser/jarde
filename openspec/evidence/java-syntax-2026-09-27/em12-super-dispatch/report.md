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

`--out` 必须是空目录。脚本现以第二片 Jarde 完整重编、验证运行 `number` 为验收；旧的编译失败保留在 `baseline/binding` 作为修前证据。编译中间物、JAR 与临时类自动删除；这里只保存可审查的文本证据。

## EM-12 修后验收

[after/summary.json](after/summary.json) 是从本变更的 Jarde CLI 和上述固定 JADX revision 重放所得。普通样本的原 class、JADX、Jarde 完整依赖源码分别通过 `javac --release 8` 和 `java -Xverify:all`，输出 `20:10:1:3`；重载样本的三侧完整源码也分别通过相同检查，输出 `number`。[Jarde 生成的 Case.java](after/binding/source/jarde/em12/Case.java) 保留 `Case.super.pick(arg1)`，同轮家族报告为 `projected`；[编译日志](after/binding/binding-jarde/javac.log)与[运行日志](after/binding/binding-jarde/runtime.log)保存第三侧结果。原 class 与 JADX 的对应日志也在 `after/binding`。

放行仅覆盖单个普通引用实参：成员方法参数描述符与目标一致，SSA 在每个调用点直接读取该形参，且所选完整 class 父链证明竞争者参数是严格子类。`NarrowArg` 静态形参即使显式转成 `Arg`、`null`/cast、局部中转、缺失 `NarrowArg` 声明和可适用的 `Object` 重载均保持拒绝；同型显式 `other` 的捕获负例仍由现有桥调用单元测试覆盖。泛型、接口、数组、varargs、装箱与 checked exception 重载仍在证明边界之外，不能据此声称通用 Java 重载恢复。
