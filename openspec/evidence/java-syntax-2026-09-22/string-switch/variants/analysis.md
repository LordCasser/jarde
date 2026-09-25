# 共享标签与穿透的边界

冻结 CLI `/tmp/jarde-cli-deferred-budget-908c` SHA-256 `908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570`。`run_audit.py` 原样编译 `StringSwitchVariants.java`（876 B，SHA-256 `234d6676b62896bb4a8865eb1bf21bed75bc9560132a2853e60ccc886c45f83d`），独立 runner 逐项测碰撞 `Aa`/`BB` 的共享返回、`x`→`y` 穿透、空串、Unicode `雪`、default、null、selector 正常/抛错及调用次数，共18行。原 class 与 JADX 整类源码均 `javac --release 8`/`java -Xverify:all` 成功，逐行 diff 空。

冻结 jarde 的 `class-source` 成功，但完整源码含3处 `@bytecode`，`javac` 报 `choose` 缺少返回语句，因此没有 jarde 运行对照，不能称值已错。javap 表明 javac 先用 `hashCode`/`equals` 选出判别值，再在 BCI 141 的整数 `tableswitch` 中让 case 0/1 共用 BCI 180，case 2 执行字段加十后落入 case 3 的 BCI 192。jarde 已正确写出第一层 hash/equals；第二层报 `SwitchArmsOverlap`，其后 BCI 192 误报重入。这里的直接缺口是**普通 switch case 穿透所有权**，与字符串分派是否折叠成一个 `switch(String)` 分开。

`present-proved-java-structure` 未完成任务 2c.4 已约定“前 case 只走到后 case 入口时，停在入口、后 case 独占块且前 case 不写 break”；本样本为其新增真实 Java 8 对照。`recover-string-switch` 的 1.2 包含共享/fallthrough：只有该层能安全认领穿透，或新的组合证明自己完整认领两级路径后，才可把此输入算为字符串 switch 正例；不能以文本替换绕开现有拒绝。
