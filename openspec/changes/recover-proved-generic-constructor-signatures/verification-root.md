# Root 独立验收：泛型构造器

结论：通过。实现保留物理成员身份，只在同轮 AST/SSA/Code 证明 `aload_0; invokespecial Object.<init>()V; return` 且正文无参数使用时投影构造器局部类型参数；投影标记准确注明 `empty-constructor proof`。reader 方法 `Signature` 的解析与擦除证明未另造副本。源码层要求顶层普通类直接继承 Object、零接口、无本类 `<init>` Methodref、无未能原位保留的注解/throws；参数槽、已发布类型变量作用域和无返回类型的构造器拼写均在完整声明发布前核对。候选只对 `<init>` 执行额外收集和预算计费，普通方法未受这一步影响。

独立重放：`python3 openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/replay.py` 退出 0；脚本逐变体断言原/JADX/Jarde 完整类 Java 8 重编、反射 `types=1`/`parameter=T`、合法显式 `<Integer>` caller 运行并保留 `T`，以及错误 `<Integer>`/`Double` caller 被 `javac` 拒绝。Jarde `-g`/`-g:none` 源码 SHA-256 分别为 `82f1b02a821f047bff791bc21b26999ada7848ae0a3d08547585b4891f6b9f4b`、`b515bfd463e8d079c2875cb660a327f02f7ac31a8004d4c0259b4923499e710e`。`negative/replay.py` 独立退出 0：未绑定变量、擦除矛盾、参数正文使用、`this(...)` 链和本类构造器调用五例均保留物理声明并通过 Java 8 重编；变造 Signature 仍可 JVM 验证。两脚本的私有 Cargo target 自动清理。

私有 `/tmp/jarde-constructor-root-target` 中运行的 Rust 测试均通过：`class_source` 47/47、`generic_constructor_projection` 4/4、`generic_method_budget` 4/4、`generic_method_projection` 9/9、`generic_throws_projection` 8/8、`ordinary_generic_projection` 14/14、`field_generic_projection` 3/3、reader `method_signature_proof` 3/3 和 lib `signature` 15/15、query lib 7/7。构造器测试含 essential/all、budget/cancel、正文拒绝与相邻方法不变。`cargo clippy -p jarde --test generic_constructor_projection -- -D warnings` 在五项既有仓库 lint 例外下通过；`cargo fmt --all -- --check`、`git diff --check` 和 `openspec validate recover-proved-generic-constructor-signatures --strict` 通过。验收后已精确清理 root 私有 target（约 1.5 GiB）。

保留的边界：带效果/委托/非 Object 父类/同类构造调用的合法泛型构造器仍使用物理声明。这里是有意的证据门，不在本次混入调用绑定或继承求解；后续覆盖应按独立三方样本扩围。
