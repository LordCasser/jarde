# 2026-09-25 独立验收记录

本 change 的正例已闭合：当前 CLI 从冻结 `Base.class` 重新生成 `Base.java`，`javac --release 8` 单类重编后仅替换原 jar 中的 Base，`java -Xverify:all` 与原 class 均输出 `observed=captured-value`、`visibleDuringSuper=true`。左假短路控制的原/Jarde 输出同为 `false-result=false,calls=0`、`true-result=true,calls=1`。普通 `?:`、断言开关与非规范 `Z` 2/3 的完整类重编/运行见 [控制复核](verification-controls.md)。这些事实证明限定两测试字段值的发射，不证明任意多前驱 Phi。

异常边负例从 [冻结 Java 8 class](../../evidence/java-syntax-2026-09-25/short-circuit-exception-edge/analysis.md) 的两测试/两生产者/单一 `putstatic` 出发。架构师独立的全证据 CLI 报告将该局部一次标成非结构化 `jre_region_exception_edge`，引用 BCI 0/1/4/7/10/11/14/15/18，catch 与后续返回仍有来源。原 class 与 JADX 左真路径 `calls=1`，Jarde 引用文本 `calls=0`；方法质量为 fallback，未声称行为相同。共享 true、非 1/0、双写入、字段身份与独立效果的 fresh recovery 拒绝矩阵见 `build.rs` 聚焦测试；真正额外入口仍是任务 1.4 未完成的边界。

当前树上已实际执行并通过：

```text
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde-java --lib                 # 161/161
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde --test p3_short_circuit_exception_edge  # 1/1
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde-java --test p3_conditional_values       # 2/2
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde --test class_source --test p3_boolean_field_stores --test p3_exception_scope --test p3_guard --test p3_nested_try  # 47+3+1+13+2；JDK 项默认忽略
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde --test p3_boolean_field_stores recovered_z_field_stores_match_the_patched_jvm -- --ignored --exact --nocapture # 1/1，40 行 JVM 对照
CARGO_TARGET_DIR=/tmp/jarde-sept25-root-proof-target cargo test -p jarde-cli --test class_source_cli            # 16/16
cargo fmt --all -- --check
git diff --check
openspec validate recover-conditional-field-writes --strict --no-interactive
```

`cargo clippy -p jarde-java --lib -- -D warnings` 实报 16 个既存 lint（enumswitch 2、region 2、report 7、build 4、reuse 1），未见本次短路新增告警；未在语义 change 中批量改写这些架构债务。`cargo test -p jarde --test p3_typed_catch` 为 4/5：`finallyIncrements` 仍因 `local 1 crosses a quoted fallback region` 失败，仓库已在 [异常局部作用域任务 2.5](../preserve-local-scope-across-exception-regions/tasks.md) 单独记录此旧缺口，它不含短路候选。本 change 的 1.4 真正额外入口仍未闭合，因此不能把整个 change 宣称完成。
