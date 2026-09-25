# 根代理验收：值渲染停止传播

实现保持一个共享 `Budget`。`lambda::plan` 在读取四段 descriptor 前按字节收费，对每个 SAM 参数适配按步收费；Builder 在创建 Local/Cast AST 前按节点数收费。值渲染以 `ValueRenderFailure` 区分普通形状拒绝与 `StopReason`，包括嵌套的 lambda/capture/调用实参路径；停止直达 `build()` 的执行平面，不再被 fallback 当作普通不支持形状。method reference 不创建 adapter AST，也不为它收这笔费用。

root 独立测得 `strings()` essential 正常用量为 33 IR、122 analysis、2500 output；`IrItems=31` 在 BCI 0 的两节点 Local/Cast 预收费处停止，已用 30 IR，正文和来源表都为空。带捕获的 `captured()` 在 BCI 5 的嵌套返回值渲染处也能以同一内部两节点阈值停止。bound-null 对照在 BCI 9 的计划工作预算耗尽时报告 `Budget(AnalysisSteps)`，没有伪装成原 `jre_lambda_sam_types`；不限制预算时仍保留该拒绝。直接进入计划的预取消精确报告 `Cancelled { at: Some(42) }`，用量为零。方法恢复 API 在更早的 IR 表阶段预取消误报 `IrTableMissing { canonical }`，是另案入口分类债务。

原/JADX/Jarde 的完整捕获正例在 Java 8 目标下重编、`-Xverify:all` 执行，四行输出一致；其中第四行由原 bound-null 控制类产生，Jarde 没有宣称恢复该拒绝形状。永久 JDK 测试再次比较原 class 和完整 Jarde 类。`p3_lambda_adaptation` 11/11（含 2 项 JDK）、`jarde-java --lib` 149/149、`p3_java_recovery` 32/32、调用实参 4/4（含 JDK）、即时函数接收者 2/2、引用 cast 6/6（含 JDK）、deferred order 3/3（含 JDK）、语料指纹 5/5。`cargo fmt --all -- --check`、三个关联 OpenSpec strict 均通过。

严格 `cargo clippy -p jarde-java --lib -- -D warnings` 在共享工作树仍报 15 项，分布在 enum/region/report/build/reuse；本轮新 `parse_method_input` 类型复杂度及通用 fallback 参数触发的 32 项告警已定点清理，故严格门禁**仍未通过**，不能记作语义验收成功。reader 历史人口断言也未重钉：当前实测 `(216,1375,128,613,8)`，旧钉值 `(164,1152,98,381,8)`。两者没有混入本语法/停止传播改动。
