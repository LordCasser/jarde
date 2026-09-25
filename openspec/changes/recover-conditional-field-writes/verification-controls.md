# 条件字段写入：独立行为控制（2026-09-25）

此记录只验收 `recover-conditional-field-writes` 的 2.1/2.2；1.4 的拒绝矩阵和最终 2.3 门禁另行闭合。架构师使用当前工作树的 `jarde-cli class-source`，均以 `javac --release 8` 重编生成源码并用 `java -Xverify:all` 执行。只有受测类由新源码替换，其余依赖取冻结原件。

## 左侧为假时不调用右侧

独立执行 `sh openspec/evidence/java-syntax-2026-09-25/short-circuit-left-false/reproduce.sh`，退出码 0。脚本重新编译 Java 8 源与冻结 class 逐字节比较，随后打 jar，执行原 class、JADX 1.5.6 完整源码和 Jarde 完整源码。三者均为：

```text
false-result=false,calls=0
true-result=true,calls=1
```

Jarde 的 `assign` 仍只有一次 `result` 写入，右侧在内层 `?:` 中；完整命令、源码、`javap` 与运行日志见[控制样本报告](../../evidence/java-syntax-2026-09-25/short-circuit-left-false/report.md)。此样本用副作用计数验证求值次数，没有单独制造 RHS 异常。

## 普通双臂 `?:` 与 boolean 字段消费

从项目根目录执行：

```sh
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde-java --test p3_conditional_values --locked --quiet
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test --test p3_boolean_field_stores --locked --quiet
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test --test p3_boolean_field_stores recovered_z_field_stores_match_the_patched_jvm --locked -- --ignored --exact
```

结果分别为 2/2、3/3（另有一项按 JDK 要求默认 ignored）及 ignored 项显式运行 1/1。另将冻结 `tests/fixtures/p3-conditional-values/v8/TernaryValues.class` 单独打 jar，用当前 CLI 的 `class-source --class TernaryValues --policy plain-jar --release 8 --format text` 得到完整 `TernaryValues.java`，连同仓库的 `TernaryRunner.java` 重编。源码中 7 个双臂 `?:`、零个 `@bytecode`；16 行输出与原 class 逐行相同，覆盖返回、赋值、算术、调用实参、引用、重载绑定以及选中分支抛出的异常和 `trace` 次序。

## 断言开关与非规范 `Z` 的 2/3 臂

把 `tests/fixtures/p3-assert-core/v8/AssertCore.class` 或 `AssertCore-non01-arms.class` 各自作为 `AssertCore.class` 打成 jar，分别从当前 CLI 生成完整 `AssertCore.java`，与 `AssertRunner.java` 以 `javac --release 8` 重编。两份生成源码均编译成功，`<clinit>` 方法报告均为 `structured/java`、`fallbacks=[]`。原件写入为 `(!AssertCore.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0`；补丁写入为 `(!AssertCore.class.desiredAssertionStatus() ? 2 : 3) % 2 != 0`。后者保留了原始整数值，并在真实 `putstatic Z` 消费处使用 JVM 最低位，没有把 2/3 推断成 Java `true`/`false`。

两份原 class 与相应 Jarde 重编 class 的 `AssertRunner` 输出逐项相同：

| class | `-ea` | `-da` |
| --- | --- | --- |
| 原 1/0 | `1\|0;bad\|2\|1` | `0\|0;0\|0` |
| 补丁 2/3 | `0\|0;0\|0` | `1\|0;bad\|2\|1` |

这些控制不证明所有异常边上的等价，也不把 JADX 对非规范 `Z` 的不可编译源码算作通过；其旧三方证据见[断言 fixture 说明](../../../tests/fixtures/p3-assert-core/README.md)。
