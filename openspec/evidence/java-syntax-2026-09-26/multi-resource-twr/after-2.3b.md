# 2.3b 后双资源头现状复核

2026-09-26 在主干 `af138e57` 用当前源码构建 CLI，对冻结 `release8/MultiResourceTwr.class` 运行 `class-source --class MultiResourceTwr --policy single-class`。`run()I` 仍是 `explanation_only`，诊断 `jre_guard_span` 的首个未解释指令为 BCI 0；另有 `jre_region_uncovered_blocks`，BCI `[43, 59, 57, 51, 73, 67]`。`new@1` 同一恢复报告称 2 个构造候选、2 个已呈现，说明缺口不在构造站点缺席。冻结字节码中的两个头部分别为 `0 new; 3 dup; 4 ldc; 6 invokespecial; 9 astore_0` 和 `10 new; 13 dup; 14 ldc; 16 invokespecial; 19 astore_1`，体自 BCI 20 开始。

代码依赖顺序：`report.rs` 先 `region::recover`；Region 遇异常边调用 `guard::examine`，在此决定 `Resource.init` 并由 `explained` 检查 `[start,join)` 的完整覆盖；之后才做 `field::plan`、`concat::plan`、`init::sites`。因此 Guard 此时没有可复用的已验证构造站点。合理接缝是前移这些独立规划并只读传入，逐项检查站点与 Store 的 SSA 值和头部范围，而非在 Guard 再写一份 `new; dup` matcher。该记录仅确定现状和依赖方向；尚未声称 2.4 已实现或完整类已可编译。
