# Root 独立验收：静态直接返回方法的局部泛型异常

结论：本 OpenSpec 首片通过。`project_method_signature` 先由 reader 解析方法 `Signature`，检查方法局部变量作用域、descriptor 及物理 `Exceptions` 的逐位置擦除；新分支只在已有同轮 `GenericReturnCandidate::Parameter`、完整静态参数/返回证明及严格类层级/异常界门成立时，把 `throws X` 与整份泛型方法头一次交给 `project_generic`。没有新 parser、正文证明机制或依赖。空正文、无正文、普通方法与构造器的选择门仍各自独立。同一个 `T` 同时用于参数、返回和异常已由旧拒绝回归改为真实 Java 8 重编及反射正例。

[三方正例](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/replay.py)在 root 独立私有 CLI 下以 `--mode recovered` 退出 0。原 class 与 Jarde 的 `-g`/`-g:none` 完整类均重编，`-Xverify:all` 执行 `ok`，泛型反射均为 `params=2, return=T, throws=X`，显式 `<String, RuntimeException>` 调用方编译通过。JADX 1.5.6 的类虽重编，但异常反射是 `java.lang.Exception`，原调用方两种配置均编译失败。Jarde 输出 SHA256 分别为 `39a4bf6e4e925cee09ce9d4bd3978cb6961699dca0c936faf39f4e4af56901b6`、`17d77eaf4a557d772e4846e942d8444bd0134f45f71a14abd22362329bf9f30b`；class 与 fixture 摘要见[分析](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/analysis.md)。

[负例](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/negative/replay.py)以 `--mode recovered` 退出 0：孤立 `EchoOnly` 正向投影；`X extends IOException`、副作用正文、包装调用、同类 Methodref 保守拒绝。变造的未绑定 `^TY;` 与 `Exception` 界/物理 `IOException` 冲突都可由 `java -Xverify:all` 装载，reader 分别以 `jvm_signature_scope_unproved` 和 `jvm_signature_erasure_mismatch` 拒绝，不能由 JVM 可装载性替代元数据证明。同类调用触发 `generic_call_binding_unproved` 后物理回退与调用点组合的整类可编译性仍是[独立债务](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis-negative.md)，未算作本项成功。原负例分析曾把擦除矛盾泛称为 `generic_source_shape_unproved`；root 逐方法检查后把断言改为实际 reader 错误码并复跑。

独立 Cargo target 定向测试通过 125/125：本项 3、静态泛型 9、类级泛型异常 3、构造器 4、普通参数化 14、完整类 47、字段泛型 3、泛型预算 4、方法局部/无正文异常 13、reader 方法证明 3 与签名单测 15、query 单测 7。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-static-method-local-generic-throws --strict` 均通过。严格 Clippy 首次只命中 `jarde-java` 中现存的 `useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref`、`collapsible_if`；仅对这五类作命令行例外后 `cargo clippy -p jarde --test static_method_local_generic_throws -- -D warnings` 通过，未发现本项新增 lint。

代理与 root 的私有 Cargo target 均已清理；root 的 `/tmp/jarde-static-throws-root-target` 清理前为约 2.5 GiB。本轮早先基线 CLI 的约 882 MiB 目标也已移除。项目目录约 217 MiB，清理后磁盘可用空间约 101 GiB。

首片仍拒绝自定义异常界、类自有泛型签名、未知父类/接口、非直接参数返回和本类调用。它以 JVM/reader/同轮 AST/SSA 来源证明决定源级声明，不复制 JADX 的物理异常集合合并。相关 [throws 顺序](../../evidence/java-syntax-2026-09-25/throws-order/analysis.md)与[内部类外部实例构造](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/analysis.md)继续独立巡查。
