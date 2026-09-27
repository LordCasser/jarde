# EM-20：局部声明、SSA 合流与同步区域后的槽位复用

固定清单项为 [EM-20](../../jadx-feature-inventory-2026-09-27/expressions-misc.md)。本次在 `origin/main` 基线 `a25f9d57b957601ad866b65e5be308a5b4577b3a` 审计，未修改生产代码或清单汇总。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；[summary.json](acceptance/summary.json) 锁定五个测试、四个生产入口、输入源码、原 class 和 Jarde CLI 的 SHA-256。Jarde CLI 由该基线源码在独立 Cargo target 构建，SHA-256 为 `165dd69000032ad8cf869331b68675181da2d5e5cc6ac7a337b1af1e8084ae1c`。

[input/em20/](input/em20/) 是合法 Java 8 完整类与共同 Runner。`joined` 让两个分支写同一个局部再读取；`loop` 在循环体内声明临时值；`caught` 让 `try` 与 catch 后的局部值合流；`synchronizedLoop` 在同步区域写入循环上界，离开同步区域后才声明、使用循环整数。这最后一项缩小了固定 `TestVariablesUsageWithLoops` 中同步区写局部、后续循环读取的作用域关系，没有把其 `List`、泛型或增强 for 变成本轮的正向主张。固定 `TestVariablesDefinitions` 主要检查增强 for 的迭代器不泄漏；`TestVariablesInLoop` 与 `TestVariablesGeneric` 走 Smali，后者禁用编译；`TestVariables4` 的多层反射/异常分支比本夹具宽。本次不宣称这些完整测试已由该 Java 8 对照覆盖。

在上述基线提交执行 `CARGO_TARGET_DIR=<临时目录> cargo build -p jarde-cli`，再运行 `python3 replay.py --jarde <临时目录>/debug/jarde-cli --jadx <固定JADX> --jadx-checkout <固定checkout> --out <空目录>`。脚本先核对固定哈希，再以 `javac --release 8 -g:none` 编译原 class，分别提取固定 JADX 与 Jarde 的**完整 `LocalScopes` 源码**，与同一 Runner 重编并执行 `java -Xverify:all`。原 class 与 JADX 重编均成功，七行输出同为 `5, 3, 0, 10, 6, 2, 3`；Jarde 完整源码重编失败，`synchronizedLoop` 在第 57 行“缺少返回语句”。两次独立重放的摘要和两份反编译源码逐字节相同。源码及编译、运行、反编译、`javap` 日志保存在 [acceptance/](acceptance/)；它们是可重放证据，不是修后验收。

前三个方法在加入同步样例前的首轮完整类对照中通过了重编与验证运行；最终 [Jarde 源码](acceptance/source/jarde-LocalScopes.java) 仍显示相同的局部形态：`joined` 将 `int local2` 放在 if 前并在两臂赋值，`loop` 将 `local3` 声明在循环体内，`caught` 把 catch 参数限定在 handler 头并让 `local1` 在外层合流。该首片的普通声明位置、整数类型与简单异常合流未发现差距。固定 JADX 把 `joined` 折为三元表达式并消去一些循环临时量；这属于 EM-26/循环呈现质量，不能当作 EM-20 作用域错误。

差距仅在同步清理后的**物理槽位复用**。[javap](acceptance/javap-LocalScopes.txt) 显示 `synchronizedLoop` 槽位 2 在 BCI 3 接收监视器引用、BCI 20 改存循环整数；槽位 3 在 BCI 14 接收异常清理引用、BCI 22 改存循环整数。`jarde-java::reuse::typed_split` 原本可用 SSA 类型和定义/使用链分割 `Reference`/`Int`，但目前遇到任意 `CanonicalEdgeKind::Exception` 就整体拒绝；同步语句的清理 handler 正好带这种边。于是 [Jarde 源码](acceptance/source/jarde-LocalScopes.java) 在两次整数写入处报告 `Object`/`int` 冲突，将后续循环和返回写成 BCI 注释；[JADX 源码](acceptance/source/jadx-LocalScopes.java) 把同步区赋值及后续循环分开，完整重编、验证运行正确。Jarde 的拒绝保持了来源和保守性，却暴露了当前首片恢复边界。

只为这个经受控对照证明的边界建立[窄 OpenSpec](../../changes/split-proved-monitor-slot-reuse/proposal.md)：借用已经恢复的同步 guard 范围验证清理异常边与两个生命周期的不可达关系，再让现有命名、类型和声明计划消费分割结果。任意 catch/finally 后的复用、Smali 专有变量合流、`TestVariables4` 的多层异常区域与增强 for 仍需独立审计；这些架构债务不混入本变更。
