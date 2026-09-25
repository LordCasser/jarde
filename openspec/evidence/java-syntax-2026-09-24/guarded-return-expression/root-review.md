# 同步返回表达式：root 独立验收

2026-09-24 在共享工作树的当前 CLI 上重新执行；实现与反例的完整字节码说明见 [analysis.md](analysis.md)。

- `Locked.java` 用 `javac --release 8 -g:none` 重编得到原 class SHA-256 `9a38d88a79ceafbf31b14f68d028ead9a50b77445ce90488a33e10da53b79c73`。当前 Jarde class-source 的 `locked` 直接写 `return this.n;`，没有 `saved0`；其 Java 8 重编 class 与原 class **逐字节相同**。原、JADX 1.5.6、当前 Jarde 的 `java -Xverify:all` 都输出 `locked:7`、`locked:-4`。
- `GuardReturnEffects` 原 class 与 JADX 重编 class 均通过 `-Xverify:all`，六行完全相同：`effect:5:15:1`、`effect-throw:after-read:17:1`、`effect-lock:released`、`nested:3:13:1`、`nested-throw:after-read:14:1`、`nested-lock:released`。当前 Jarde 对独立调用及未证明的嵌套 monitor 保留带真实 BCI 的 fallback；没有把这两种形状误写为可编译的直接返回。负例的 Jarde 输出有意不作 Java 重编。
- 独立 Rust 回归：`p3_sync_return` 3/3、`p3_guard` 13/13、`p3_loop_transfers` 4/4、`p3_switch_fallthrough` 2/2、`p3_deferred_value_order` 2 通过/1 既有 ignored、`p3_twr_catch` 1 通过/1 既有 ignored。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate preserve-guarded-return-expression --strict` 均通过。

所有权边界直接审读 `guard_return_ownership` 与 `Builder::build`：前者仅从已证明的 `Shape::Monitor` 取 normal exit、return 与 body，重复 return key 撤销豁免；遍历中每个 Region 都 `poll`/`charge`，map 仅在完整成功时返回。后者在建立 Builder 前用 `?` 接收整个 map，因此预算或取消中断不会发布部分所有权。没有另写只镜像这个 `Result` 控制流的低限单测。`has_independent_boundary` 只在 producer、最终 return、body 和该退出 BCI 同时吻合时略过这一次指令；相邻值放置与 Guard 测试检验其它效果仍是边界。
