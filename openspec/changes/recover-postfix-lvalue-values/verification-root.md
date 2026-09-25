# Root acceptance — Java 8 postfix old-value returns

2026-09-25，root 在共享 worktree 中独立审查 `PostfixUpdates::prove`、`prove_postfix_field`、`prove_postfix_array`、`Builder::postfix_expression`、AST/emitter 的后置节点和来源遍历。证明要求写回和 `ireturn` 相邻、字段 `I` 的 owner/name/descriptor 与接收者复制相同，或已证明 `[I` 的数组/下标复制相同；SSA 单消费者把旧值送返回、新值送 store；生产者填满可延期前缀且不跨独立效果。旧简单字段规则仍优先。候选才建立按 `(canonical block, BCI)` 索引的 handler effect 表；成功链的每个锚点与返回有序 handler 集相同，且预算计费/取消轮询有界。

## 整类可执行对照

冻结正例为 1,051 B、9 个 `Code` 方法、SHA-256 `7548934ba19c23d533517a76560d525011f8ec261c8a6670b46c1860314cbe21`；root 从相邻 Java 源码重编得到逐字节相同的 class。root 用本轮构建的 CLI 输出原样完整 Jarde 源码，并用本地 JADX 1.5.6 完整输出；JADX 自己生成 `defpackage`，仅把自写 runner 放进相同 package，不编辑两种反编译源码。原 class、JADX、Jarde 三者均以 `javac --release 8 -g:none` 编译、以 `java -Xverify:all` 运行成功，17 行输出完全相同。输出涵盖字段旧值 41/新值 42、数组旧值 70/新值 71、调用轨迹 `R`/`AI`、null 与界限异常、`Integer.MAX_VALUE` 回绕，以及已有简单字段前/后置对照。

合法跨 handler 负例为 1,219 B、SHA-256 `0999c796238fed342fc2210e2ee7ac20b2200615662704fe28cd86f1b554cf1b`；root 重放相邻 `rebuild.py` 后哈希仍相同。原 class 在 `-Xverify:all` 下输出 `escaped=IllegalArgumentException` 与 `calls=2`。Jarde 目前在更早的 Region 阶段以 `jre_guard_resource_init`/`jre_region_uncovered_blocks` 拒绝，没有错误输出 `return a()[i()]++`；因此此样本验证未误折叠，不能当作后置 handler 检查直接被执行的证明。Region fallback 只映射 BCI 0/13 的逐指令来源缺口已单列 [OpenSpec](../preserve-region-fallback-instruction-origins/proposal.md)。

## 回归与界限

- `jarde-java --lib`：143/143。`p3_postfix_lvalue_values`：4 个常规 + 2 个显式 JDK 测试全过；八个 JVM 验证的身份/消费者/独立效果负例均拒绝，普通赋值没有误变成后置更新。来源覆盖正例调用、复制、读、加、写、返回；`all`/`essential` 正文相同；输出预算与预取消无半成品；32 层接收者触发既有深度拒绝而未泄漏 `++`。
- 相邻测试：`p3_postfix_handler_boundary` 1/1、`p3_field_increment` 2/2、`p3_compound_lvalue_updates` 4/4、`p3_array_access` 11/11、`p3_deferred_value_order` 常规 2/2 及显式 JDK 1/1。`p5_corpus_fingerprint` 在纳入两个新 source/class 对后 5/5（576 文件）。
- `cargo fmt --all -- --check`、`git diff --check`、本变更及独立 Region/lambda 规划的 `openspec validate --strict` 均通过。严格 `cargo clippy -p jarde-java --lib -- -D warnings` 被 15 条既有跨文件 lint 阻挡，后置新增区间没有诊断。

未将相邻既存失败算作后置回归：`p3_prefix_survival` 两条旧断言要求 fallback，而当前已输出完整 `while`/`if`；reader census 旧钉值 `(164,1152,98,381,8)` 对当前真实 `(214,1369,128,613,8)`。两者已在 roadmap 拆出，不能通过扩大本次语法规则或无依据改钉值处理。
