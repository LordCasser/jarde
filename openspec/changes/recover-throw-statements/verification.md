# 验证记录

## 实现边界

普通 `athrow` 现在由 `StmtKind::Throw { value }` 表示。`Builder::instruction` 从该指令的
真实 SSA stack reads 取唯一异常操作数，并在最终 `athrow` BCI 调用既有 `render_value`；栈中
更低、随后被 `athrow` 丢弃的值不会被当成 throw 的读取。`new`、`invoke` 和 `checkcast`
的已有消费/失败生产者路径把 `Throw` 视为真实 reader，因此表达式只呈现一次，guard 已拥有
的合成重抛仍由 guard 区域负责。

发射器增加 `throw <expr>;`，并把 `Throw` 视为 switch arm 的终止语句，不追加不可达
`break`。来源仍来自既有 statement/expression origins；未请求 source map 时不构建它，预算
停止通过既有 commit/replay 事务丢弃未支付的 throw 或 source-map 节点。实现没有新增 pass、
索引、缓存、Throwable 层级猜测或强制 cast。

## 固定输入与边界

正面输入是 `tests/fixtures/p3-throw/v8/ThrowProbe.class`：946 bytes、major 52、9 个
带 `Code` 的方法，SHA-256 为
`0b7a0e314fc3c2ee0ecd2bd226033aa735374d9456bc502bfba0ac7b5845e8a3`。它覆盖 `null`、参数、
`new`、调用结果、显式 cast、条件两臂、命名 catch 和 checked exception；完整 class 的
JDK 对照没有删除 guard/source-only 方法。

`tests/p3_throw.rs` 用冻结 class 的完整 `method_info` 做三种临时内存 patch，不增加永久
fixture：

- stale parameter patch 在 `aload_0` 后改写局部值；恢复结果要么保留旧 stack value，要么
  在真实边界处 Mixed/Fallback，若正文出现 `arg0 = null;` 则禁止错误的 `throw arg0;`。
- duplicated call patch 插入 `dup/pop`；调用结果可以被完整引用，但生产者最多执行一次。
- lower stack patch 先执行带副作用的 `ThrowEffects.problem()`，再让 `athrow` 读取参数；
  测试确认副作用位于唯一 `throw arg0;` 之前且各出现一次，清栈不产生额外读取。

三种变体均使用 `java -Xverify:all` 的边界日志；相关 class digest 和日志在
`openspec/evidence/java-syntax-2026-09-22/throws/refusal-boundaries/`。finally、synchronized
及其 guard 合成重抛仍保持既有拒绝边界，本 change 没有扩大这些区域。

## 本地验证

| 检查 | 结果 |
| --- | --- |
| `cargo test -p jarde-java --lib --locked` | 100 passed / 0 failed；含 switch Throw 终止、throw 输出预算和 source-map budget replay 单元测试 |
| `cargo test --test p3_throw --locked` | 7 passed / 0 failed / 1 ignored（加入三种边界和精确 stale/lower 断言后复跑） |
| `cargo test --test p3_throw --locked -- --ignored` | 1 passed / 0 failed；完整正面 class 的原始/恢复源码经 JDK `-Xverify:all` 执行结果一致 |
| `cargo test --test p3_guard --locked` | 13 passed / 0 failed；guard 重抛所有权和 monitor/TWR 边界未回归 |
| `cargo test --test p3_new_value --locked` | 4 passed / 0 failed；`new` 消费规则未回归 |
| `cargo test --test p3_invocation_arguments --locked` | 3 passed / 0 failed / 1 ignored；调用参数规则未回归 |
| `cargo build -p jarde-cli --locked` | 完成；`/tmp/jarde-throw-cli-final.log` |
| `rustfmt --edition 2024 --check`（throw 生产文件及 `p3_throw.rs`） | 通过 |
| `openspec validate recover-throw-statements --type change --strict --no-interactive` | Change valid；`/tmp/jarde-throw-openspec-validate.log` |

对应日志保留在 `/tmp/jarde-throw-final-tests-with-boundaries.log`、
`/tmp/jarde-throw-final-jdk.log`、`/tmp/jarde-throw-jarde-java-lib-budget-final.log`、
`/tmp/jarde-throw-p3-guard.log`、`/tmp/jarde-throw-p3-new-value.log` 和
`/tmp/jarde-throw-p3-invocation-arguments.log`。

## Root独立验收（2026-09-23）

root审读Throw操作数获取、最终求值位置、构造/调用消费链、来源遍历、emitter及switch终止，未增加guard准入或层级猜测。重建后的CLI恢复完整ThrowFlow，15项构造器、嵌套调用、数组、字段及条件场景均为0引用、原样javac通过、执行与原class相等；JADX也通过，只有source-only异常类的自动包前缀在观测比较中被正规化。证据为`throws/independent-flow/after/`，没有删除或替换生成方法。

root从最终Rust测试直接提取三个method_info补丁，再次独立执行-Xverify:all：stale保留原异常身份且引用BCI 3/0；dup/pop保留独立生产调用并引用3/4/5；lower保留一次problem()，仅NOP BCI 3引用，随后throw参数。对应证据在`throws/refusal-boundaries/root/`。三者都是明确的Mixed边界，未将可编译的引用正文当作完整执行恢复。

`throws/grammar/`另外确认全throw switch无多余break、完整javac通过。两个全路径抛异常的clinit在jarde和JADX都因Java初始化器正常完成规则编译失败，已独立记录声明上下文债务；不扩张本项。

| Root检查 | 实际结果 |
| --- | --- |
| `cargo test -p jarde-java --locked` | 178通过：100 unit、32 recovery、46 patterns |
| throw/eval/new/guard/cast/numeric/invocation/D1/fingerprint | 61通过，JDK等显式ignored不计入 |
| `p3_throw -- --ignored` | 1通过，完整冻结正面类13行执行相等 |
| reader全fixture遍历 | 91 class / 509 Code / 75 handler / 231 branch或switch target / 8 subroutine，通过 |
| fingerprint | 219文件；新增14，原205无变动，无删除；5项检查通过 |
| 受影响文件rustfmt | 通过 |
| 全仓OpenSpec strict | 37项通过、0失败 |
| `clippy -p jarde-java --all-targets -- -D warnings` | 停在既存region.rs:1736 type_complexity，无新allow；不声称全部lint目标已完成 |

上述命令和统计证据已归档在`throws/root-regressions/`。fixture增量只含throw、final-static、instanceof三个永久class，其helper/runner为source-only；后续语料增量另行冻结，不改写本次测量结论。strict clippy既存债务、finally/synchronized guard扩展、clinit全异常出口和接口初始化仍独立处理。
