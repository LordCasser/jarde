# Root 独立验收

验收对象是方法类型变量遮蔽类类型变量的受限声明恢复。Root 复核 [`signature.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-reader/src/signature.rs)：方法参数名只在方法同层检查重复；方法作用域在擦除查询中先于已证类作用域，方法第一界、参数、返回和 `Exceptions` 都走同一查找；未绑定和循环界仍拒绝。此次无 `class_source` 新候选、AST 或依赖改动，现有静态直接参数返回门在 reader 证明成功后自然投影。对无界与异界的旧证据，JADX 1.5.6 仅无界版本能满足原 `StrongCaller`，不能把其异界输出当正确基线。

Root 从共享源码独立运行：

- `CARGO_TARGET_DIR=/tmp/jarde-nested-audit-target cargo test -p jarde-reader signature::tests`：17/17，通过同层重复、未绑定、自循环、类 `T`/方法 `T` 异界、类 `T`/方法 `U`、参数/返回/`Exceptions`、错擦除、预算和预先取消。
- `CARGO_TARGET_DIR=/tmp/jarde-nested-audit-target cargo test -p jarde-reader --test method_signature_proof`：3/3。
- `CARGO_TARGET_DIR=/tmp/jarde-nested-audit-target cargo clippy -p jarde-reader --lib -- -D warnings`、`rustfmt --edition 2024 --check crates/jarde-reader/src/signature.rs`、`openspec validate recover-shadowed-method-type-parameters --strict`：均通过。`rustfmt` 最初仅提示该文件顶部两项 import 顺序，Root 已按工具格式化，未改逻辑。

独立 [`accept-jarde.py`](../../evidence/java-syntax-2026-09-25/method-type-variable-shadowing/accept-jarde.py)在 `-g` 与 `-g:none` 下编译原始两类及 `StrongCaller`，分别用当前 Jarde CLI 导出完整 `ShadowPlain`/`ShadowBounded`；`essential` 与 `all` 的 Java 文本逐字相同且报告执行完成。只用导出的两份类、未改的 `StrongCaller.java` 和临时反射 harness 做 Java 8 重编，`java -Xverify:all` 两轮均打印 `plain`、`bounded`，类/方法变量的首界、方法参数和返回泛型反射文本均与原 class 一致。该脚本对实现前 CLI 曾以 `Object`/`CharSequence` 到 `String` 的两处 javac 错误失败，修后通过。

Root 又构造了真实 classfile 的[错擦除负例](../../evidence/java-syntax-2026-09-25/method-type-variable-shadowing/analysis-negative.md)：仅把 `ShadowBounded.echo` 方法 `Signature` 的局部界由 `CharSequence` 改成 `Number`，不改物理 descriptor/Code；patched class 在 `-Xverify:all` 下仍输出原两行。新 reader 以方法 `T` 的错误 `Number` 擦除对照 `CharSequence` 物理参数，在参数 0 局部报 `jvm_signature_erasure_mismatch`；Jarde 不发布伪泛型方法头。有无调试表均重放并核对 SHA 清单。此拒绝比“只让类/方法同名通过”更关键：后者会把错误声明伪装成 Java 源。

`class-source` 是多成员入口，没有单一 driver BCI。Root 额外试 `--evidence source_map --evidence-bci 0..2`，报告按既有合同返回 `partial/jre_evidence_range_invalid`；这不是本次签名逻辑停止。方法级范围证据与 class-source 的范围支持属于其它入口合同，本项只对该入口支持的 `essential/all` 做正文等同性断言。并行 generic enclosing change 仍在编辑 `class_source`；它完成后需再跑上述整类脚本作共享树回归。

代理自己的 `/tmp/jarde-shadowed-method-agent-target` 已清理。Root 将旧三方探针 CLI 按 SHA 相同复制为 `/tmp/jarde-cli-audit-baseline` 后，使用 `cargo clean --target-dir /tmp/jarde-nested-audit-target` 清掉该私有编译目录，实际移除 1.7 GiB 文件；只保留约 42 MiB 的重放用 CLI 副本。
