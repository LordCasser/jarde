# 2026-09-26 全量测试基线审计

在 `6360f94d^` 的干净源码归档中，用独立 `CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0` target 复跑当前 `cargo test --locked --all-features -p jarde --no-fail-fast` 所报的 14 个失败 target。基线已有 13 个 target、17 个失败测试；当前全量运行中这 17 项仍失败。它们不属于本次 do-while 或命名成员家族调用实现的回归，不在对应语法切片内调整断言或生产逻辑。

| 基线失败 target | 失败测试数 | 观察到的边界 |
| --- | ---: | --- |
| `p3_finally_straight` | 1 | 旧断言要求拒绝已呈现的分支式 `try/finally`；原 class 与当前源码的 Java 8 四路执行对照相同。 |
| `p3_hoisted_boolean` | 2 | 旧拒绝等级和固定账本不符。 |
| `p3_lambda_adaptation` | 2 | 预算边界断言不符。 |
| `p3_meeting`, `p3_method_ir`, `p3_numeric_comparison`, `p3_required_conversions` | 各 1 | 各自的旧拒绝、IR 账本或呈现边界不符。 |
| `p3_shift_expressions`, `p3_shift_negative_boundaries`, `p3_throw` | 各 1 | 预算、负例拒绝或值来源断言不符。 |
| `p3_try_local` | 2 | 资源/局部声明与旧文本断言不符。 |
| `p3_type_qualifier` | 1 | 名称呈现旧断言不符。 |
| `p5_bulk_corpus` | 2 | 固定计数账本随已有恢复行为变化。 |

当前并行全量运行还报 `p3_immediate_functional_receivers::captured_array_length_stays_a_lambda_call_and_keeps_its_physical_helper`：`javac` 运行时找不到测试刚写的临时源码。在当前工作区将整个 target 以 `--test-threads=1` 重跑为 6/6；干净当前 HEAD 的并行复跑中，此项通过，但同一 target 的另一 Java runner 临时 class 未找到。该 target 的并行临时文件问题须单独诊断，不能据一次结果归因于反编译语义或本次 Region 改动。

`jarde-java --test p3_java_recovery::a_member_that_cannot_be_presented_leaves_the_member_that_can_alone` 在当前与 `6360f94d^` 干净归档中均于第 1838 行失败；两份 `RecoveryReport` 去掉 `elapsed_millis` 后逐字相同，旧断言把非确定性耗时当成了输出合同。该测试也应在独立测试维护切片处理。

本次变更的正面门禁另见 [do-while root 验收](../../../changes/recover-do-while-body-transfers/verification-root.md) 和 [命名成员调用证书验收](../../../changes/assemble-proved-member-class-family/verification-3.2.md)。本文件只记录基线债务，不改变任何恢复规则或测试。
