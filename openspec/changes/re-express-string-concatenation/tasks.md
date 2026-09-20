## 1. 复现与证据（修正前完成）

- [ ] 1.1 T4 反例先行：受控样本 `new StringBuilder().append(a).append(b).append("!")`，记录恢复正文 `arg0 + arg1 + "!"`、报告平面、以及**两侧都能编译但值不同**的事实（输入 `(1, 2)`：原 `"12!"`、文本 `"3!"`）（A13）。
- [ ] 1.2 T1 反例先行：按复核的形状（`new StringBuilder().append(s)` 重复 N 次 + `.toString()`）在**修正前**的 debug CLI 上记录 N = 1024（完成）、1536 与 2048（exit 134、空 stdout、`has overflowed its stack`／`stack overflow, aborting`）；默认与放大预算各跑一次，确认 abort 与预算无关（A13、A14）。
- [ ] 1.3 定位与深度证据：在当前源码上核对 `build.rs::concat_expr` 的左折叠（约 2586–2595）与 `emit.rs` 的二元臂（475–479，复核记 476），用 `lldb --batch -o run -o bt`（或等价方式）取一次 native 栈，记录重复帧与所在文件行，确认崩溃在发射而不是释放；记录每层帧距或等价的栈代价测量，并写明构造/发射/克隆/释放四条路径各自的深度来源（A13、A14）。
- [ ] 1.4 构建边界记录：分别在 debug 与 release 上记录同一形状的完成/失败范围（release 主线程对 2048–15000 的复核结果如实重测或引用），写明两条是不同构建边界、都不是界的依据，也不得相互替代（A14）。

## 2. 实现

- [ ] 2.1 引入片段表示（design 决策 1）：每个片段带自己的表达式、该 `append` 的参数类型与 origin/BCI 锚点；`concat_expr` 按 `chain.appends` 迭代把片段放进序列，MUST NOT 再折成 `Box<Expr>` 链；片段内部嵌套仍受 `MAX_VALUE_DEPTH`（A13、A14）。
- [ ] 2.2 打印机按片段序列迭代（`emit.rs`），逐个片段按自己的上下文与嵌套打印；核对构造、发射、克隆（`ExprKind` 派生 `Clone`）与释放四条路径都由序列长度界定，并把每条的实测写进 verification（A13、A14）。
- [ ] 2.3 字符串上下文起始（design 决策 2）：按首个 `append` 的参数类型决定是否在序列前插入空字符串片段（既有 `ExprKind::Str("")`）；插入片段的 origin MUST 由链的既有 BCI 派生，段表 MUST NOT 出现一条它没有执行的指令（A12、A13）。
- [ ] 2.4 片段拼写按参数 descriptor（design 决策 3）：`boolean` 片段的 `0`/`1` 字面量写成 `true`/`false`；`null` 与对象片段按其 overload 的转换写出；片段值是局部读取时读 [unify-local-type-decisions](../unify-local-type-decisions/design.md) 的局部类型决定，MUST NOT 在此再作一次局部类型判定（A13）。
- [ ] 2.5 顺序与不变量核对：片段序列顺序即 `chain.appends` 顺序，求值次数/次序与异常次序不变；MUST NOT 平衡、合并或重排片段；`keeps_its_conversion` 的接受集合与拒绝路径逐字不变（由 4.1/4.6 与 5.2 的既有回归核对）（A12、A13）。

## 3. 受控 fixture 与语料登记

- [ ] 3.1 提交转换样本 `tests/fixtures/p3-concat-conversion/`：`ConcatConversion.java`（两个数值在前、需要转换的片段在最后、全字符串、`append(boolean)`、`null`、对象片段、「一个片段本身是加法」（`append(a + b).append("!")`）、一个可观察/可能抛异常的片段）、`v8/ConcatConversion.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码、修正前的恢复正文与两侧值）、`Baseline.java`（A13）。
- [ ] 3.2 深链 fixture 由**测试内生成器直接产出字节**，不用 `javac -J-Xss64m`：链长是检查的参数（复核区间与更大规模都要跑），生成器才能确定地给出它；编译器放栈是产出手段、不是可进入回归的依赖。生成器写成两份逐字节相同的副本（`tests/p3_eval_context.rs` 的库入口与 `crates/jarde-cli/tests/task_cli.rs` 的子进程入口），沿用 `recursion_fixture` 的注释约定（点名镜像的复现与忠实性判据）；若 frame pass 需要属性，按 `recursion_fixture` 补最小 `StackMapTable` 并写明原因（A13、A14）。
- [ ] 3.3 语料登记（只对 3.1 的真实样本）：在 `tests/fixtures/README.md` 的 P3 real compiled samples 表新增一行；执行 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过，记录文件数变化（A13）。
- [ ] 3.4 更新 reader fixture census：`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections` 元组按实测更新并显式记录；内存生成的深链 MUST NOT 被硬造一行登记（A13）。

## 4. 回归、对照、子进程检查与变异

- [ ] 4.1 常备回归（精确文本）：新建 `tests/p3_concat_conversion.rs`，断言关键成员的文本在第一个数值片段处进入字符串拼接、布尔片段是 `true`/`false`、「一个片段本身是加法」保持自己的分组（`"" + (arg0 + arg1)`）而「两个独立片段」不被合并，以及全字符串/「数值在最后」两条对照逐字不变；修正前该用例 MUST 在文本断言上失败。门禁：`cargo test --test p3_concat_conversion --locked`（A13）。
- [ ] 4.2 编译与执行对照：把转换样本登记进 `tests/p3_execution_comparison.rs` 的 `REQUIRED`，输入集合覆盖多顺序、boolean 的两个取值、`null` 与非 `null` 对象、「加法片段 / 两个片段」这对分组形态（`(1, 2)` 分别 `"3!"` 与 `"12!"`）、以及可观察/抛异常片段；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`（A12、A13）。
- [ ] 4.3 常备回归（库入口，深链）：同一生成字节经 `Engine::recover_method` 得到报告（能呈现时为文本与 content 分类，不能时为既有停止），断言不含 signal 语义的失败；重复运行稳定（A13）。
- [ ] 4.4 子进程检查（debug，默认门禁）：在 `crates/jarde-cli/tests/task_cli.rs` 的子进程模式里运行 `jarde-cli recover`，覆盖**正常完成**与**预算中途停止**（用 `--budget DIMENSION=LIMIT` 让构建/发射途中断）两种情况：断言 exit 为既有成功或「执行未完整」状态、stdout 是 JSON 报告、无 signal/exit 134/空 stdout；MUST 在默认线程栈上运行，MUST NOT 设置 `RUST_MIN_STACK` 或使用大栈线程（A13、A14）。
- [ ] 4.5 子进程检查（release，显式门禁）：以 `#[ignore]` 的显式门禁构建 optimized CLI（`cargo build --release -p jarde-cli --locked`）并运行 4.4 的同一断言集合，记录两个构建各自的退出状态与 stdout 摘要；release 侧结论 MUST NOT 代替 debug 侧证据（A13、A14）。
- [ ] 4.6 变异（两个方向）：(a) 把片段表示退回左深 `Binary` 折叠，重跑 4.3/4.4，debug 子进程检查 MUST 因 signal/abort 变红；(b) 移除字符串上下文的起始（恢复从第一个原始值左折叠），重跑 4.1/4.2，对照 MUST 在 `(1, 2)` 上以值不同变红。MUST NOT 用其它用例的红代替；恢复后 `grep` 无调试残留、文件哈希复原（A13、A14）。

## 5. 门禁与收尾

- [ ] 5.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录输出摘要与 ignored 数量（A13）。
- [ ] 5.2 相关既有回归：`cargo test -p jarde-java --test p3_patterns --locked`（`concat@1` 的识别与既有拒绝）、`cargo test --test p3_eval_context --locked`、`cargo test --test p3_boolean_contexts --locked`、`cargo test --test p5_corpus_fingerprint --locked` 通过，确认接受集合、既有拼接形状、类型拼写与语料指纹未被本次修正改变（A13、A17）。
- [ ] 5.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（T1 的 signal 与 native 栈、debug/release 两个边界、build/emit/clone/drop 四条路径的实测、T4 的两侧值、按参数类型的片段拼写、anchor 处理、fixture 摘要、census/fingerprint 计数、多顺序与 boolean/null/对象对照、子进程 debug/release 结果与预算停止清理、两个变异、门禁结果）；未完成前不归档、不更新完成判断。
