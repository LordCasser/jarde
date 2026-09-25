# 调用参数的静态类型丢失导致重载错值

这是独立于普通 checkcast 和 super 接收者选择的 P1。源码、javap、两种反编译文本及三份执行结果均保存在本目录。javac 23.0.1 `--release 8 -g:none`；jadx 1.5.6；jarde 使用已含 special 接收者修复的 debug CLI。jarde 整类输出没有字节码引用，直接编译成功，但三个方法均返回错误值。

| 方法 | 源码关键点 / CP 目标 | 原 class | jarde 重编译 | jadx 重编译 |
| --- | --- | --- | --- | --- |
| superObject | `super.pick((Object)"x")` / `(Ljava/lang/Object;)I` | 1 | 2 | 2 |
| ownObject | `own((Object)"x")` / `(Ljava/lang/Object;)I` | 3 | 4 | 4 |
| primitive | `number((int)c)` / `(I)I`，c 声明为 char | 7 | 8 | 7 |

这三种源码转换没有独立 JVM cast opcode：引用向上转换不需要 checkcast，char 与 int 共享 JVM int 栈类型。返回/赋值时能隐式转换，不代表调用时可以丢弃参数的静态类型，因为源码会重新进行重载选择。jadx 只在 primitive 例补出 int cast，另两例也错。原 class 始终是执行基线。

## 已确认的构造路径

`Builder::arguments` 把调用与赋值/返回一起交给 `meeting_position(..., Widening::Position)`。数值 widening 在 Position 下不写 cast，而 `conversion` 对引用类型直接视为 Same。于是符号目标的 descriptor 虽仍在事实中，却不影响编译器看到的实参静态类型：`"x"` 选择 String 重载，char 选择 char 重载。

后续独立 change 应将“参数能传给该 descriptor”与“生成源码仍选择该 descriptor”区分。可以复用既有 Cast、Type 和已解码方法 descriptor，按调用位置保留所需静态类型；不需要新的 AST 或 resolver，也不应为了枚举外部重载而读取 callee Body。对当前三个反例，`(java.lang.Object) "x"` 和 `(int) arg1` 已足够。来源应注明这是调用位置提供的呈现类型，不能伪称原字节码存在 checkcast/i2i。

完整方案尚须核实 null、数组、多个参数、构造器、boxing 与 lambda/方法引用的目标类型；不能用 `presented=None` 继续默认安全，也不能在未知类型上随意添加可能改变异常行为的检查。这些验证完成前不将候选策略标为实现任务完成。赋值/返回无需因此普遍添加 cast，避免把特定调用约束扩散到所有位置。

`OverloadEdges.java` 与 edges-javap.txt、edges-jarde.java.txt 是随后补充的边界输入。后续 Luna xhigh 已在 `boundaries/` 完成逐例原 class/jarde/jadx 编译执行对照；主代理核对了九个独立例子，所有 jarde 编译输入均与 CLI stdout 逐字一致，没有替换方法体。

| 边界 | 原 class | jarde | jadx |
| --- | --- | --- | --- |
| null → Object | 1 | 2 | 1 |
| String[] → Object | 3 | 4 | 3 |
| Integer.valueOf → Object | 5 | 6 | 5 |
| System::nanoTime → Runnable | 7 | 8（选择 Supplier） | 7 |
| Object/Object 两参数 | 1 | 2 | 1 |
| Object 构造器参数 | 1 | 2 | 1 |
| byte/short 常量参数 | 1/3 | 2/4（选择 int） | 1/3 |
| Supplier lambda 正面对照 | 8 | 8 | 8 |
| Runnable lambda | 7 | 编译失败，synthetic 方法名冲突 | 7 |

最后一项是既有 lambda 声明呈现问题，不借本修复重做 synthetic 方法管理。整类 OverloadEdges 同样保留该真实 javac 失败；没有删掉冲突方法后声称整类成功。byte/short 结果同时证明：调用上下文不能借用赋值上下文的常量窄化；`onlyByte(3)` 不是 `(byte)3` 的等价调用。后续规格需要修正旧 2c.29 对调用参数“无指令则不写 cast”的过宽裁决，同时保持赋值与返回原有规则。

`type-boundaries/` 已完成额外执行：外部泛型调用原值1，jarde/jadx均为2；去掉 Runnable checkcast 的 class 通过 `-Xverify:all` 且接收new Object返回7，补cast则会新增CCE。不能仅以 `presented == required` 排除重载风险，也不能将参数 descriptor 当作任意引用类型可安全 checkcast 的证明。独立 `preserve-invocation-argument-types` 已据此完成规划；安全同型、Object、null、数值与已证明函数式目标之外的引用关系保守拒绝，无新resolver。规划完成不代表实现完成。

源码层选择依据见 [JLS 15.12.2](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.12.2)。当前子任务 preserve-special-call-dispatch 已明确不处理重载/bridge，其验收只覆盖接收者分派类别；本问题单独排队，不能用已修 super 掩盖。它也不属于 recover-explicit-reference-casts，因为这些例子根本没有 checkcast。

## 重放

```sh
mkdir -p /tmp/jarde-overload-replay
javac --release 8 -g:none -d /tmp/jarde-overload-replay OverloadParent.java OverloadProbe.java OverloadRunner.java
java -cp /tmp/jarde-overload-replay OverloadRunner
```

随后以同一 OverloadProbe.class 调用 jarde 的 class-source / single-class / release 8，将完整输出作为 OverloadProbe.java，与未修改的父类和 runner 编译执行。jadx 输出仅移除自动添加的 `package defpackage;` 以匹配原默认包。执行结果为 original.txt、recovered.txt、jadx-result.txt。
