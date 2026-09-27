# DT-29 完整类族：当前主线基线

[重放脚本](../replay.py)用主线 `df65f687` CLI（SHA-256 `7f8dafe1d928570cde099a10807a395ca209ea2e54f5479fd6295d99ec9dc92f`）重编冻结的五个输入源，核对九个物理 class 与旧快照 SHA-256 完全相同，检查固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 及两个测试、三个实现文件的 hash。加入泛型反射门槛后的独立复跑目录为 `/tmp/jarde-dt29-main-reflection-IVbCXfqX`；[summary.json](summary.json) 和当前 Jarde 的 [根类](FieldCast.java)、[B](FieldCast$B.java)、[C](FieldCast$C.java)、[D](FieldCast$D.java) 源码已归档。

原始与固定 JADX 的完整源码均以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行，输出 `runnable:1111:0000:1111:ClassCastException`；单独反射 Runner 均输出 `1:T:dt29.FieldCast$B:T:dt29.FieldCast$B`，同时约束类型变量、上界、泛型实参和物理擦除。Jarde 的九个物理类源码整体重编失败：根类 `run` 和 `bits` 分别在第 28、49 行缺少返回语句。B 的父字段写入现已完整；剩余失败精确分布如下。

| 方法 | 当前拒绝位置 | 机制缺口 |
| --- | --- | --- |
| `FieldCast$C.set(B,Z)` | 2、7、12、17 | B 实际接收者到 A 字段 owner；B 实参到 `A.access$002(A,Z)Z` |
| `FieldCast$D.set(B,Z)` | 2、7、12、17 | 同上；另有 `<T extends B>` 方法头因非平凡 void 正文被拒 |
| `FieldCast.run()` | quoted 起点 17、34、83，真实调用 BCI 14、31、74 | 三次 `B→A` 私有 `bits(A)` 调用实参缺证 |
| `FieldCast.bits(A)` | quoted 起点 0、21、38、55、72、78 | 四段条件值跨块进入 `StringBuilder.append(String)`，现有 carried-conditional/concat 组合未闭合 |

这组基线分别对应里程碑 P1、P2、P3；任何一个单包输出方法片段都不足以宣布九类源码已可编译。重放脚本写入独立输出目录，不覆盖旧组合审计快照，且校验原始与 JADX 语义后才记录 Jarde 的失败状态；集成时加 `--require-jarde` 会同时强制运行输出、泛型反射值及零 `@bytecode`。
