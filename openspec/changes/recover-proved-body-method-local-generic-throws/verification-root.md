# Root 独立验收：空正文方法自有泛型异常

结论：通过。`project_method_signature` 继续先由 reader 解析方法 `Signature`，证明局部变量作用域、方法 descriptor 及物理 `Exceptions` 的逐位置擦除；随后新分支只消费已有同轮 `GenericReturnCandidate::EmptyVoid`。该候选要求唯一 `return` 的 AST/Code/SSA、无 handler/phi/读写/抛出效果且 BCI 一致，不能从最终文本倒推。源码门另限顶层普通类、直接父类 Object、零接口、无类 `Signature` 属性、无参数 `void`、单个 Throwable 根界的 `<X>` 和同名 `throws X`，并拒绝 Object 同名方法及本类同名 Methodref。实际类属性是否存在单独传入：类泛型头拒绝时交下来的空作用域不会误认作“非泛型类”。

无正文泛型方法和本次空正文方法共用最终方法头拼写器；原无正文、类级 `throws E`、静态参数返回与构造器路径未放宽。该拼写器只包含当前使用的参数槽名称，没有预留的可选参数名机制。正文分支不另算一遍擦除；完整 `<X> ... throws X` 经一次 `project_generic` 发布，标记指明 `same-run AST/Code/SSA empty-void and method-local Signature scope/erasure proof`。

独立正例重放 `python3 openspec/evidence/java-syntax-2026-09-24/body-method-local-generic-throws/replay.py` 退出 0：原 class 与 Jarde 在 `-g`/`-g:none` 下均完整 Java 8 重编、`-Xverify:all` 运行，反射为 `parameters=1`/`throws=X`，显式 `<RuntimeException>` 调用方编译并打印 `called`。JADX 1.5.6 的类可重编，但反射退化为 `throws=java.lang.Exception` 且调用方编译失败。两种 Jarde 源码 SHA-256 均为 `03a56795e61d917ebc52714bddb3b225bbb41c7f2c459ecaf9269b41dfe337e3`。

独立负例重放 `python3 openspec/evidence/java-syntax-2026-09-24/body-method-local-generic-throws/negative/replay.py` 退出 0：未绑定的 `^TY;` 与第一界/物理异常擦除矛盾两份变造 class 均通过严格 JVM 验证，Jarde 分别以 `jvm_signature_scope_unproved`、`jvm_signature_erasure_mismatch` 拒绝；合法非空正文也拒绝本首片，三份物理回退完整类可重编。同类调用以 `generic_call_binding_unproved` 先拒绝，但其物理回退完整类因原泛型调用绑定而无法重编；这是[独立后续债务](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis-negative.md)，没有计入可重编负例。Rust 定向测试另以真实异常表和 verifier-valid 的 `finalize` 改名证明 handler、Object 同名拒绝，覆盖类 `Signature` 存在但可发布作用域为空的变造、essential/all 文本相同以及预算/取消时无部分方法头。

独立私有 Cargo target 的定向 Rust 测试通过：`generic_throws_projection` 13/13、`class_source` 47/47、`generic_body_throws_projection` 3/3、`generic_constructor_projection` 4/4、`generic_method_budget` 4/4、`generic_method_projection` 9/9、`ordinary_generic_projection` 14/14、`field_generic_projection` 3/3、reader `method_signature_proof` 3/3 与 lib `signature` 15/15、query lib 7/7。严格 Clippy 首次检查只命中既有模块五类 lint 和本变更新增的一处 `len_zero`；新增项已修正，在仅对既有五类 lint 作命令行例外后 `cargo clippy -p jarde --test generic_throws_projection -- -D warnings` 通过。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-body-method-local-generic-throws --strict` 均通过。

两套重放脚本及独立巡查的[异常顺序重放](../../evidence/java-syntax-2026-09-25/throws-order/replay.py)都使用自动清理的私有 Cargo target。root 私有 `/tmp/jarde-method-local-root-target`（约 1.8 GiB）验收后已精确删除；项目共享 `target`（约 959 MiB）也已清理。最终仓库目录约 216 MiB，当前磁盘可用空间约 95 GiB。

首片仍不处理带参数/非空效果正文、自定义异常界、类泛型、继承/接口契约或跨类调用绑定。`throws` 顺序以及内部类外部实例构造另按[当日巡查](../../evidence/java-syntax-2026-09-25/throws-order/analysis.md)和[内部类证据](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/analysis.md)拆分，不混入此实现。
