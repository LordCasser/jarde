# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。反例来自 benchmark 战役，根因由实施者用原生栈定位。CI 结果随后回填。

## 反例与根因

**反例**（单 class、默认预算、公开 CLI；class 取自 `S2-007.war` 的 `WEB-INF/lib/javassist-3.11.0.GA.jar`）：

```text
recover --input javassist/bytecode/CodeAnalyzer.class --policy single-class \
  --class-name javassist/bytecode/CodeAnalyzer --method-name computeMaxStack --descriptor '()I'
→ 修正前：exit 134，stdout 0 字节，stderr 「thread 'main' has overflowed its stack / fatal runtime error: stack overflow, aborting」
→ 修正后：exit 4，stdout 是报告
```

**根因（原生栈证据，非推断）**：`lldb` 回溯前 200 帧是同一三帧循环的 66 次重复，每层约 26 KiB：

```text
region_at (region.rs:892) ← latch_tested_loop (region.rs:1601) ← loop_region (region.rs:1383) ← region_at (region.rs:926) ← …
```

**驱动是循环而非深度**：临时深度计数（已还原）显示走查越过 300 层后才崩溃，且每层都重入**同一状态**（`node=Some(1) bci=37 boundary=Some(4) scope_len=Some(6) header=Some(true)`）。机制：对「latch 在别处、header 块不是测试」的 latch-tested 循环，`latch_tested_loop` 从**循环自己的 header** 开始走查循环体，而 `region_at` 会把「带着空前缀进入的循环 header」读成「从外部进入的循环」并派发到 `loop_region`，其形状检查不读 frame，于是同一调用无限递归。位置是 `crates/jarde-java/src/region.rs`，与 `jarde-jvm`/`jarde-reader` 无关。

## 修正

`Walker::region_at` 改为受保护的入口（内层 `region_at_inner`），**检查在下降之前**，覆盖该递归族的唯一入口（各 arm、两种循环体、每个 switch arm）：

| 码 | 触发 | 位置 |
| --- | --- | --- |
| `jre_recursion_bound` | 进入会超过 `MAX_REGION_DEPTH = 32` 层嵌套 | 被拒进入的块 |
| `jre_recursion_reentry` | 进入的块正是当前帧自己正在构建的循环的 header（`Frame::own_loop`） | 即将重入的块 |

两者都走**既有** `StopReason::Interrupted`（无新变体、无新平面、无新预算维度）。`Frame::own_loop` 只由 `Frame::loop_body` 设置、由 `Frame::arm` 清除，因此合法的 `continue` 回到外层循环测试仍走原路径——重入拒绝只可能对可证不终止的循环体走查触发。报告层 `report.rs::stopped` 增加按码措辞；既有 `jre_budget_interrupted` 的文案逐字节未变（码迁到命名常量）。

**上限值 32 有测量依据**：已提交语料 308 次走查最大 **2** 层；`javassist-3.11.0.GA.jar`（347 class / 3398 方法）3277 次完成的走查最大 **17**（20 次 ≥10）；每层约 26 KiB，32 层 ≈ 850 KiB（debug 下低于测试线程 2 MiB、远低于主线程 8 MiB）。

**同时修掉的一处守卫失效**：`render_value` 的嵌套调用/实参边（`call_expr` 的 receiver/实参、`invoke_expr` 的 accessor receiver、`concat_expr` 的片段）此前以 `depth = 0` 回调，使既有 `MAX_VALUE_DEPTH` 守卫只覆盖到这些边为止；现在 `depth` 贯穿并传 `depth + 1`。算术渲染未触碰（那是相邻 change 的范围）。

## 修正后的报告（我独立复跑，同一输入）

```json
"outcome": "performed",
"presentation.execution": {"status": "partial", "reason": {"kind": "error", "code": "jre_recursion_reentry"}},
"presentation.content": "not_produced", "presentation.quality": "fallback",
"recovered.recovery.text": "" (0 字节), diagnostics: [jre_recursion_reentry … at BCI 37]
usage: analysis_steps 512, class_headers 2, method_bodies 1, result_items 27
```

**这是如实拒绝而不是截断**：没有任何产物被提交（`content = not_produced`、无文本、无段表），原因同时点名重入与递归界，位置给出块，退出 4 是既有的「执行未完整」。`outcome` 为 `performed` 是对的——目标选择已完成（唯一），未完成的是**执行**，由 `presentation.execution` 陈述。

## 回归、变异与正向对照

- **受控 fixture**：`tests/p3_eval_context.rs::recursion_fixture()` 与 `crates/jarde-cli/tests/task_cli.rs::recursion_fixture()` 在内存中构造 `p/Recursive.method()V`（两份 `cmp` 逐字节相同，170 字节，sha256 `226d57b4…8f8e20a`），字节来自 `javac 23.0.1 --release 8 -g:none` 对 `int x=0,i; do { for (i=0;i<3;i=i+1) x=x+1; } while (x<5);` 的输出加 javac 自己的 `StackMapTable`（v52 体需要它，否则 frame pass 会以无关的 `jre_ir_table_missing` 拒绝）。形状与真实驱动一致：latch-tested 循环（header 不是测试 ⇒ `header_tested_loop` 退让）、header 块结束处正是内层 `goto` 目标开始处（内层回边，正如真实 `computeMaxStack`）、latch 独占一块、纯 latch 测试。深度是「一层循环」，不是人为叠层。
- **修正前证据只记录一次、不断言**：在修正前二进制上（`/tmp/jarde-repro/jarde-cli-prefix`）该文件给出 exit 134、stdout 0 字节与上述栈溢出。测试断言的是**界的行为**，不是崩溃。
- **变异**：删掉 `region_at` 的守卫检查 → 恢复无界递归，两条回归以**信号**变红（`fatal runtime error: stack overflow`, `(signal: 6, SIGABRT)`；CLI 侧 `the command exited with a status rather than a signal`）。变异已还原（`grep -c MUTATION` = 0，文件哈希复原）。
- **正向对照（真实语料，逐字段对照，仅剔除 `elapsed_millis`）**：`javassist-3.11.0.GA.jar` **3397/3398 相同、0 差异**，1 个修正前 abort（`CodeAnalyzer.computeMaxStack()I`）现在给出报告；S2-009 的 `antlr-2.7.2.jar` **2297/2299 相同、0 差异**，2 个修正前 abort（`StringUtils.stripFront/stripBack`）现在以同一 `jre_recursion_reentry` 停止。无关停止保留自己的原因：预算/取消停止用例仍绿，`jre_budget_interrupted` 的码与措辞未变，语料 113 处 `ir_table_missing` 前后一致。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test p3_eval_context --locked` | 8 passed / 0 failed |
| `cargo test -p jarde-cli --test task_cli --locked` | 12 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1256 passed / 0 failed / 6 ignored**，连跑两次一致（基线 1254，+2 新回归） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（17.6 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `openspec validate --all --strict --no-interactive` | 22 passed / 0 failed（归档前） |

改动文件：`crates/jarde-java/src/{region,stop,report,build}.rs`、`tests/p3_eval_context.rs`、`crates/jarde-cli/tests/task_cli.rs`。未新增 crate/依赖/线程，未使用 `catch_unwind`（栈溢出无法捕获），守卫是显式的下降前检查。

## 边界与残留

- **深度臂未被单独触发**：没有测试输入到达 32 层，`jre_recursion_bound` 是后备臂；其停止**形状**由重入臂与既有预算/取消停止证明。要单独覆盖它需要 >32 层嵌套 while 的生成器（可行，本次刻意未写——abort 证据指向重入臂，而 33 层输入按设计会被拒）。
- **`latch_tested_loop`（latch ≠ header）现在整次运行拒绝**而非局部化到该循环：这是设计明确的「停止而非局部化」取舍，且在两个真实语料上它恰好只作用于原本会 abort 的方法。
- **嵌套调用链深于 `MAX_VALUE_DEPTH` 现在会被拒（fallback）**，此前会渲染；在 5697 个真实方法上未观察到影响，也未影响语料结果。
- 既有弱点、本次未动：`emit::expr` 对 `concat_expr` 构造的左折叠递归，其深度等于一条拼接链的 append 数（实践中受 `code_bytes` 约束，无 abort 证据）——属发射器形状工作，留给相邻 change 的范围。
