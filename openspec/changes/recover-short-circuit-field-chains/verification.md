# 2026-09-25 独立验收

[冻结三方对照](../../evidence/java-syntax-2026-09-25/short-circuit-chain-shared-true/analysis.md)先固定了实现前错误，再由架构师用当前 CLI 重新生成完整类和全证据 JSON。`ChainOrField.assign(ZZ)V` 现在仅有一次字段写入，无引用字节码；`quality=structured`、`representation=java`、`fallbacks=[]`，source map 覆盖 BCI 0/1/4/5/8/11/14/15/18/19/22。原源码、JADX 1.5.6、Jarde 完整类各自 `javac --release 8` 重编；同一 Runner 在 `java -Xverify:all` 下的四行值与 RHS 调用次数逐字相等。JADX 对无包名输入写的伪 `package defpackage;` 只在重编副本中去掉，没有修改类体。

既有两测试 shared-false Base、shared-true OR 和左假 AND 均从当前 CLI 重新生成完整类并重编，在 JVM 验证下与原 class 逐字同输出；40 行非规范 `Z` 控制通过。[三测试真实异常边控制](../../../tests/fixtures/p3-conditional-values/short-circuit-exception-chain/README.md)由架构师独立逐字节重建、JVM 验证五条路径；当前 CLI 把 BCI 0/1/4/5/8/11/14/15/18/19/22 整组引用，catch 与后续来源可追，`quality=fallback`，不虚构等价 Java。三测试链的非 1/0 producer 补丁内部测试完整引用；[第二消费者的 Java 8 class](../../../tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true-controls/README.md)由架构师独立重编、class 逐字节相等、四路径 JVM 验证通过，其 fresh recovery 把 13 个必要 BCI 全部映射与引用且不发布结构化写入。

当前工作树实际门禁：

```text
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde-java --lib --test p3_conditional_values --test p3_short_circuit_chain_controls --locked  # 163+2+1 通过
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde --test p3_short_circuit_exception_edge --test class_source --test p3_boolean_field_stores --test p3_exception_scope --test p3_guard --test p3_nested_try --locked  # 1+47+3+1+13+2 通过，JDK 项默认忽略
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde --test p3_boolean_field_stores recovered_z_field_stores_match_the_patched_jvm -- --ignored  # 1 通过
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde-cli --test class_source_cli --locked  # 16 通过
CARGO_TARGET_DIR=/tmp/jarde-sept25-final-target cargo test -p jarde --test p3_short_circuit_exception_chain --locked  # 1 通过
cargo fmt --all -- --check  # 通过
git diff --check  # 通过
openspec validate recover-short-circuit-field-chains --strict  # 通过
```

严格 `cargo clippy -p jarde-java --lib -- -D warnings` 仍有 16 个既存 lint，位于 enumswitch、region 的 loop/try、report、build 的其它方法及 reuse；新增链式证明位置没有告警。`p3_typed_catch` 的 `finallyIncrements` 旧失败归 [异常局部作用域任务 2.5](../preserve-local-scope-across-exception-regions/tasks.md)，未纳入这次链式语义改动。

本轮验收的私有 `/tmp/jarde-sept25-root-proof-target` 在清理前 `du` 为 3.9 GiB；`cargo clean` 删除 21,191 个文件（Cargo 计 6.0 GiB），项目根目录无 `target/`。同一数据卷的 `df -h` 可用量从 91 GiB 升至 95 GiB。实现与控制子代理的私有 Cargo target 也已各自清理。

任务 2.1 尚未闭合：三测试链的独立测试效果尚需控制。[纯 Java 8 额外入口反例](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)已冻结，`(gate ? extra : left) || other || rhs()` 的 BCI 8 `goto` 给共享真值生产者 BCI 25 增添一条链外入边。普通 region 路径四次误报 `jre_region_loop`、漏 BCI 8/26/29 来源，并将未发射的 BCI 30 字段写入标作 `presented=true`，故不能宣称该负例已安全拒绝。原/JADX/Jarde 完整类均可编译并执行 32 路径，但 JADX 与原类不同 16 行，Jarde fallback 不同 30 行；JADX 的条件极性合并在这里也不可靠。已把来源采集债与所有权/报告债分开记录，不能把修前者冒称修后者。超过表达式深度的**已闭合**链会整体认领后完整引用，不发布部分表达式。当前 OpenSpec 保持未归档。
