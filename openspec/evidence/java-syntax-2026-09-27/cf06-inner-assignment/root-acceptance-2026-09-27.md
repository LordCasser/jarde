# CF-06 主线独立验收

root 审阅 `18feeb82`：只为现有已证条件测试建立同块 `source; dup; local store; test` 的单次消费证书，采用最小 `ExprKind::LocalAssign` 呈现赋值表达式；最终 AST 必须实际消费每个隐藏的 copy，否则整方法引用。词法声明沿原计划，未改循环 Region/Frame。额外消费者、错类型、交错副作用、异常边、未被结构条件持有的候选，以及预算/取消均有定向拒绝。

root 在当前主线重新构建 CLI（SHA-256 `8e51db790172a460686da999cb241bcc6fbcbdcae86558b310ba150bdcefff89`），独立运行固定 [replay.py](replay.py) 到 `/tmp/jarde-cf06-root-acceptance`。原 class、固定 JADX 和 Jarde 的**完整类源码**均以 Java 8 重编并经 `java -Xverify:all` 输出七行 `-1,4,-1,true,true,false,false`；Jarde 源码 SHA-256 `fda8596207c3348174824239d6fc066afce93421628b1f80cc57a7f78b34c59f` 与 agent 后验收证据一致，无 `@bytecode`。

主线定向 `p3_inner_assignment` 4/4、相邻 `p3_short_circuit_chain_controls` 1/1、`p3_boolean_short_circuit_return` 2/2、`p3_mixed_short_circuit_local` 5/5 均通过；`cargo fmt --all -- --check`、`git diff --check`、OpenSpec strict 通过。全工作区另有独立测试债务；CF-06 的其它赋值位置仍待扩验。
