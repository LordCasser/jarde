## 2.4b 独立验收（2026-09-24）

root 用保留的 CLI `/tmp/jarde-catch-field-replay/jarde-cli`（SHA-256 `26d990b353e466b39e88059502c2aaa302a009bf07701f25a8b689bb0d431dfa`）对冻结 `CatchAfterFieldAssignment.class`（SHA-256 `473216d184d6b64f71c3d7b37510aa2240d44dcf61424cc545c98e3fdce7f4ae`）重跑 `class-source --policy single-class --release 8 --evidence source_map`。字段与局部方法均完整恢复，没有 `@bytecode`；字段赋值在 `try` 前恰好出现一次，BCI 2 指向 `field = 7`，BCI 12 指向命名 catch 和其字段赋值。

root 原样重编源 Java，所得 class 与冻结输入逐字节相同；将 JADX 1.5.6 输出仅去掉其附加的 `package defpackage;` 后，原源码、JADX、Jarde 三份**完整类**均用 `javac --release 8 -g:none` 编译，并在同一 Runner 下通过 `java -Xverify:all`。三者逐行均为 `field=7,local=7` 与 `field=18,local=18`。

负例只把冻结类唯一的异常表 `[5,9)->12` 起点从 BCI 5 改为 BCI 2，让保护范围从 `putstatic` 本身开始；新 class SHA-256 `e9885ad302ce4d4330927528b89a7282c41b1bc14c60264bb28968fb06cb218d`。原变体通过 `java -Xverify:all` 且两行结果不变；新 CLI 仍以 `jre_guard_resource_init` 拒绝整个字段方法，没有用户 `catch`。现有扩围 TWR 的 `p3_guard` 13 项亦通过。`p3_typed_catch` 的 `finallyIncrements` 旧失败归异常作用域 2.5，未混入本项。
