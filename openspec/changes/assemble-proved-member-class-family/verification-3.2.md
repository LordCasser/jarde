# 3.2 家族内构造调用证明

`member_family.prepared.calls` 是独立于 3.1 `capture` 的证书状态。只在双向 `InnerClasses` 关系、唯一 child 物理定义、完整 root/child 物理报告和捕获构造器都已证明后，扫描两份家族方法中的每个 child `new`。每个候选在同一请求预算内重新进入现有 `new@1`；只接受其 `presented` 结论，再核对分配 CP Class、调用 CP Methodref 是否精确绑定已证 child 构造器，和连续 qualifier/load/dup/`Objects.requireNonNull`/pop 的物理 BCI。证书按 caller 物理方法 ID、分配/副本/check/构造调用 BCI 和普通参数产生 BCI 命名。`new@1` 负责 SSA 实例与 qualifier 的两份同一来源、单消费者、参数依赖与效果顺序及异常处理器覆盖。家族 caller 只限已证 root/child，因此可使用非 public 的成员与构造器；不以 raw `ACC_PUBLIC` 推断成员可访问性。任何 body 未恢复、分析/规则未闭合、预算停止或候选拒绝，都会使整体 calls 为 `refused`，保留已证明的前缀站点与每个可定位拒绝站点。没有候选且所有方法完整扫描时，空 sites 才是闭合结论。

现有 `new@1` 类型比较原本只接受描述符参数的 `LOuter;`，而冻结正例 `main` 局部 `outer` 来源于 `new Outer`，SSA 保留的是裸内部名 `Outer`。本任务在 `jarde-java/src/init.rs` 仅放宽为目标 Outer 的**精确**内部名或精确对象描述符；其他 SSA、null、参数、效果、异常门不变。冻结真实实例给出新形式的正例；既有 `member_call_proof_requires_same_qualifier_and_early_check` 中不同静态类型、错 SSA 身份、晚 check、异常范围仍拒绝。此处不接受任意引用兼容或父类赋值。

| 样本/检查 | 证据等级 | 结果 |
| --- | --- | --- |
| [冻结阶段一完整 JAR](../../evidence/java-syntax-2026-09-26/named-member-family-stage1/README.md) | 原始 Java 8 class，`java -Xverify:all` 运行及完整类报告 | 包级 `Member` 与构造器的 root `main` 调用在 BCI 27/33/37 获证；child 构造器与物理报告不变。 |
| [限定构造与前置效果](../../evidence/java-syntax-2026-09-26/named-member-family-calls/README.md) | javac Java 8 class，`java -Xverify:all` 执行；生产 class-source 集成测试 | `internal` 的 `mark` 依赖在 check 后；`before` 的 `mark` 在分配前，构造表达式只取其已存局部值。原执行的 null/non-null 效果序列为 `P` / `PIP`。两调用点均获证且 BCI 不互相混合。 |
| 同目录 `null-check-removed.jar` | 从冻结 class 单条定长 opcode 变体，JVM verifier-valid；生产 class-source 集成测试 | BCI 27 拒绝，`main` 的物理方法与 child 报告保留。 |
| 同目录 `wrong-relation.jar` | 从冻结 class 单个 `InnerClasses` 索引变体，JVM verifier-valid；生产 class-source 集成测试 | child 自身关系不同，整个 family 拒绝；不授权包级调用。 |
| 调用证明 `method_bodies` 预算止点 | 请求集成测试 | 3.1 捕获仍 `proved`，calls 为 `refused`，根 `Partial`、child `Complete`，两份原文本与方法表不丢。 |
| 既有 `new@1` wrong identity / late check / handler controls | 冻结 verifier-valid class + 规则 proof-unit；未重新包装为本家族 | 规则层继续拒绝，不能当成本家族集成正例。 |

本阶段只建立局部调用证书。没有改写调用表达式、构造器、捕获字段、方法文本或 source map，也不宣称完整家族源码可编译。3.1 的六指令捕获构造器范围仍然适用；带初始化主体的合法成员构造器会保守拒绝。

实现侧验证：隔离 `CARGO_TARGET_DIR` 下 `cargo test --locked -p jarde --all-features --test member_family_identity` 8/8，`cargo test --locked -p jarde-java --lib member_call_proof_requires_same_qualifier_and_early_check` 1/1，`cargo test --lib member_inner::tests::family --features test-support` 3/3；`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate assemble-proved-member-class-family --strict` 均通过。三个证据 JAR 已分别用 `java -Xverify:all` 执行；运行输出保存在同目录 `*-verify.txt`。隔离 Cargo target 已 `cargo clean`。

Root 独立复核三个 JAR 的 SHA-256、`java -Xverify:all` 输出与所存日志一致，并重跑家族调用 8/8 和现有 `new@1` 精确 qualifier/early-check 1/1。源码审读确认 `init.rs` 只接受目标 Outer 的裸内部名或同一目标的对象描述符；家族调用以物理方法、BCI、已证明的嵌套关系和原有 `new@1` 闭包绑定，拒绝路径保留物理报告。全量 Cargo 回归另随最终验收记录补充。
