# Root 独立验收：有正文类级泛型异常

结论：通过。源码审读确认 `GenericReturnCandidate::EmptyVoid` 是现有 class-source 候选的受控变体，只在完整 Program 的单条 `Return(None)` 与精确单条 `return` Code、无异常表、单块无 phi SSA、唯一 `Operation::Return`、无读写/抛出效果且各自 BCI 对齐时生成。reader 继续独占方法 `Signature` 解析及异常逐位置擦除证明；源码层只允许顶层普通类、直接父类 Object、零接口、非 Object 同名方法、无参数 `void`、已发布类变量且 first bound 为确定 JDK 异常根。本类同名 Methodref、注解/flag 及声明作用域门继续生效。修饰符、异常和方法声明仍经既有 `ordinary_parameterized_declaration` 结构化拼写，再由 `project_generic` 原子发布，来源标记准确注明 `AST/Code/SSA empty-void proof`；普通参数返回候选门未被收紧。

独立正例重放 `python3 openspec/evidence/java-syntax-2026-09-24/body-generic-throws/replay.py` 退出 0。原 class 与 Jarde `-g`/`-g:none` 的完整类均以 Java 8 重编、`java -Xverify:all` 运行，反射返回 `throws=E`，`BodyThrows<RuntimeException>` 强类型调用方重编并运行。JADX 1.5.6 两种变体都写 `throws Exception`，其类能重编但反射退化、调用方编译失败。最终 Jarde 源码两种变体 SHA-256 均为 `59ce417557143ccd5b852d6e599239b67b9622c40387f0771781b0373a6c0a9b`。

独立负例重放 `python3 openspec/evidence/java-syntax-2026-09-24/body-generic-throws/negative/replay.py` 退出 0：真实 `throw`、异常处理器、本类 Methodref 均局部拒绝，物理 `throws Exception` 保留，完整类 Java 8 重编；未绑定与异常擦除矛盾的两份 Signature 变造 class 均严格 JVM 验证通过，Jarde 仍拒绝投影。继承样本在更早的父类 generic scope 门拒绝，不误计为正文门。Rust 定向测试另外覆盖 `Object.finalize()` 同名覆写拒绝、essential/all 一致、预算 Partial/MethodBodies 与取消状态。

独立私有 Cargo target 中的 Rust 测试全部通过：`class_source` 47/47、`generic_body_throws_projection` 3/3、`generic_constructor_projection` 4/4、`generic_method_budget` 4/4、`generic_method_projection` 9/9、`generic_throws_projection` 8/8、`ordinary_generic_projection` 14/14、`field_generic_projection` 3/3、reader `method_signature_proof` 3/3 与 lib `signature` 15/15、query lib 7/7。`cargo clippy -p jarde --test generic_body_throws_projection -- -D warnings` 在五项既有 lint 例外下通过；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-body-generic-throws --strict` 通过。两套 replay 的临时 Cargo target 自动清理；root 私有 `/tmp/jarde-body-throws-root-target`（约 1.6 GiB）验收后精确清理。

首片边界仍是无参数、`void`、空效果正文和类级异常变量。方法自有 `<X> throws X`、参数或非空正文、自定义异常界、继承/接口契约以及跨类调用绑定单独分析；[方法自有变量的三方探针](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis.md)已确认下一个独立缺口。
