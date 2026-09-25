# 接口字段初始化式：完整类与字段表顺序边界

`InterfaceInitProbe.java` 是 Java 8 合法接口：`CONSTANT` 来自字段自身 `ConstantValue`，其余三个字段的初值在 `<clinit>` 中依序计算，`InitEffects` 留下每次调用的轨迹。以 `javac --release 8 -g:none` 编译；冻结 CLI 为 `/tmp/jarde-cli-member-root-accepted`，SHA-256 `19ede7a6fe88b95530637e9765c7ae02b9f233520dc6a2e2e815f0dca58d0552`；JADX 为 1.5.6。运行时执行本地自写样例，不是产品自动运行目标代码。

| 输入 | class SHA-256 | 原 class 与 JADX（均整类 Java 8 编译及验证） | Jarde 整类 Java 8 编译 |
| --- | --- | --- | --- |
| 原始字段表 | `e04abe561a505d9022039776b8b29de22e35f6d4dba45a08f19a9b5c07afb527` | `ABT\|A1\|B2\|4\|7` | 失败：三个接口字段缺 `=`，且接口不允许 `static {}` |
| 交换 FIRST/SECOND 的 field_info | `01f953662dc388e8682740967e007bfeaa5671576329f87a75b0989254470fb6` | `ABT\|A1\|B2\|4\|7` | 同样失败；字段声明顺序已变为 SECOND、FIRST，而 `<clinit>` 仍先写 FIRST |

`run_audit.py` 只交换两个完整 `field_info` 记录，保持常量池、方法表与全部 Code 字节逐字节相同；补丁后的 class 由 `-Xverify:all` 执行，输出不变。JADX 在两种输入上都重建了 `FIRST=next("A")`、`SECOND=next("B")`、`TOTAL=total(FIRST,SECOND)` 的源码顺序；Jarde 则只按字段表陈列无初值声明，另列 `static {}`。脚本在独立 `/tmp/jarde-interface-init-root-s3A4Di` 重放后，`summary.json` 与此处逐字节相同。命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/run_audit.py \
  --cli /tmp/jarde-cli-member-root-accepted --out /tmp/jarde-interface-init-replay
```

这条缺口不是字段 `ConstantValue` 的再读取，也不是普通类 blank static final 赋值的去限定问题。Java 8 [JLS §9.3.1](https://docs.oracle.com/javase/specs/jls/se8/html/jls-9.html#jls-9.3.1)要求接口每个字段有初始化式，并允许非编译期常量；初值在接口初始化时按对应执行轨迹求得。字段表本身不证明初始化顺序。尚未证明多分支、额外效果、异常表或字段重写的源等价；规划必须先以完整 `<clinit>` 的身份、顺序、消费及效果作为准入条件。
