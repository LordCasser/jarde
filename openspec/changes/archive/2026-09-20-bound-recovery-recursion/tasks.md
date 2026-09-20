## 1. 复现与定位

- [x] 1.1 在固定基线上用受控输入复现进程 signal：记录完整命令、exit 状态、stderr 的 `stack overflow, aborting` 行与 stdout 为空的事实；用 `lldb --batch -o run -o bt`（或等价方式）取得 native 栈，记录重复帧、承载递归的函数与所在 crate。证据是栈顶重复帧加复现命令，不得用方法名或 crate 外观代替（A13、A14）。
- [x] 1.2 逐一核对 design Context 列出的候选递归族的现有界与递归边传递（`region_at`、`collect_guards`/`collect_paths`、`Builder::region`、`emit::expr`、`render_value` 及其重置 `depth` 的边），标出哪些边没有界；在 default 与放大预算下各复现一次，确认 abort 与预算无关（A14）。
- [x] 1.3 写出界的数值依据：复现达到的深度或循环重入点、既有语料实测最大深度、所选常量与余量。MUST NOT 只把 `MAX_VALUE_DEPTH` 或任意数字推广；结论与证据进 verification（A13）。

## 2. 实现

- [x] 2.1 在定位到的递归入口实现「进入前」的显式界检查；界内行为不变，超界返回 `Interrupted { code, at }` 经既有停止路径发布（`Stopped`、Partial、Error 诊断带 code 与位置）；不新增 `StopReason` 变体、预算维度或平面；同层同类递归共用同一个界检查 helper（A13、A14）。
- [x] 2.2 让 1.2 标记为无界的递归边也受限（例如把重置 `depth` 的调用/参数边改成传递并检查），并以变异证明这些边现在也被界拦截；不得只加一个更大的常量（A13）。
- [x] 2.3 核对停止契约不变：停止报告 text/段表为空、`content = not_produced`、usage 为本次真实值；既有预算/取消停止用例在 `cargo test --workspace --all-targets --all-features --locked` 中保持通过（A14）。

## 3. 受控 fixture 与回归

- [x] 3.1 在既有测试落点新增内存生成的受控类：具名生成器函数构建 1.1 定位证据指明的驱动形状，注释点名它镜像的递归族与复现事实，并说明不提交第三方字节的理由；生成器形状必须让**修正前**的同一字节以 signal 结束（修正前构建单独记录一次作为前置证据，不作为常备断言）。忠实性判据：不得堆叠与驱动无关的指令（A13、A14）。
- [x] 3.2 常备回归（库入口）：同一生成字节经 `Engine::recover_method` 得到 `Stopped`、非 Complete `execution` 与点名界的诊断；重复运行稳定；用例落点优先复用 `tests/p3_eval_context.rs` 的恢复夹具，需要独立目标时新建并在 verification 记录。门禁：该用例所属目标的 `cargo test --test <target> --locked`（复用时为 `cargo test --test p3_eval_context --locked`）（A13）。
- [x] 3.3 常备回归（进程入口）：沿用 `crates/jarde-cli/tests/task_cli.rs` 的进程模式在子进程内运行 `jarde-cli recover`，断言 exit 4、stdout 为 JSON 报告、报告 execution 非 Complete；signal、exit 134 或空 stdout 即失败。门禁：`cargo test -p jarde-cli --test task_cli --locked`（A13、A14）。
- [x] 3.4 变异：移除 2.1/2.2 的界（恢复无界递归）后重跑 3.2/3.3，必须变红且失败现象是 signal；恢复变异后树中不遗留调试改动，并记录变异前后两次运行（A13、A14）。

## 4. 门禁与收尾

- [x] 4.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录输出摘要与 ignored 数量（A13、A14）。
- [x] 4.2 界内结果未变的对照：`cargo test --test p3_eval_context --locked`、`cargo test --test p3_execution_comparison --locked -- --ignored`、`cargo test --test p5_corpus_fingerprint --locked` 通过并记录；语料未新增文件时不改 fingerprint，新增文件时按既有再生成流程处理（A13）。
- [x] 4.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（复现、native 栈、界依据、两层回归、变异、门禁结果），同步受影响的公开状态与引用；未完成前不归档、不更新完成判断。
