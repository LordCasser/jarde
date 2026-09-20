## Why

一次可由公开入口复现的恢复请求让进程以 signal 结束，而不是返回报告。复现以默认预算经公开 CLI 运行（standalone class，类从 `S2-007.war` 的 `WEB-INF/lib/javassist-3.11.0.GA.jar` 提取）：

```sh
jarde-cli recover --input javassist/bytecode/CodeAnalyzer.class --policy single-class \
  --class-name javassist/bytecode/CodeAnalyzer --method-name computeMaxStack --descriptor '()I'
```

观察：**exit 134**，stderr 为 `thread 'main' has overflowed its stack` 与 `fatal runtime error: stack overflow, aborting`，stdout 为空；没有报告、没有 `Partial`、没有诊断。benchmark 在七个 struts WAR 上的 `CodeAnalyzer.computeMaxStack` 与 S2-009 的 `antlr/StringUtils.stripFront`/`stripBack` 上观察到同一 abort，默认与放大预算下都出现：这不是预算不足或单个语料特例，而是恢复层的递归没有界，且失败无法以任何一种既有停止被发布。

本 change 只修正这条路径，不重开任何已归档的停止语义缺口。

## What Changes

- 先定位本次 abort 的递归族（region/CFG 遍历、表达式渲染或嵌套表达式处理）并记录 native 栈证据；在进入递归前检查显式深度界，而不是等待栈耗尽。
- **行为变化**：界内能完成的递归不变；不能完成的递归结束为已发布停止——`outcome = Stopped`、非 `Complete` 的 `execution`（Partial/Cancelled）、点名递归界与位置的诊断——全部复用既有停止词汇与平面；调用方不再收到 signal 而不是报告。
- 把该输入纳入永久回归：受控 fixture 由内存生成（不提交第三方字节），先证明修正前同一字节以 signal 结束，再证明修正后进程以报告回答；恢复无界递归的变异必须让回归变红。

## Capabilities

### New Capabilities

无。不新增 crate、公共模块、报告平面或停止分支。

### Modified Capabilities

- `java8-recovery`：恢复递归有界；界外结束为已发布停止，进程不再因恢复请求 abort。
- `recovery-validation`：把「修正前以 signal 结束的输入」纳入验收，并规定证据是报告的停止平面与退出状态，而不是「没有打印 stack overflow」。

## Impact

影响恢复层可达的递归入口与其停止路径；具体文件由定位证据决定，当前候选包括 `crates/jarde-java/src/build.rs`、`crates/jarde-java/src/region.rs`、`crates/jarde-java/src/emit.rs` 与 `crates/jarde-java/src/stop.rs`；若 native 栈证明溢出发生在 `jarde-jvm` 或 `jarde-reader`，界加在该处并在验证中如实记录。与 [fix-nested-arithmetic-value](../fix-nested-arithmetic-value/proposal.md) 都涉及 `crates/jarde-java/src/build.rs`，两者必须**串行**实施，不能并行。

非目标：不新增 crate 或依赖（含 `stacker` 一类增长栈的库）、不升级依赖、不引入 verifier、不声称一般语义等价；不把 region/SSA/AST 通用改写成迭代实现；不重开 R8/R9 或已归档的停止传播修正；不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务；不改变既有停止、取消与预算语义，不新增 `StopReason` 变体或预算维度。历史归档与既有验证记录保持原状。当前仅完成修正规划，实施任务全部待办。
