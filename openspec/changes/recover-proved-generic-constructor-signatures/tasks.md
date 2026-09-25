## 1. 冻结三方与拒绝证据

- [x] 1.1 重放顶层空正文泛型构造器的 `-g`/`-g:none` 原/JADX/Jarde 完整类、反射、合法和错误显式类型实参调用方；以 [replay.py](../../evidence/java-syntax-2026-09-24/generic-constructor-signatures/replay.py) 退出码 0、class/source SHA 与 [analysis.md](../../evidence/java-syntax-2026-09-24/generic-constructor-signatures/analysis.md) 验证。
- [x] 1.2 增补仍可 JVM 验证的未绑定/擦除矛盾 Signature、参数参加正文、`this(...)` 链和本类构造器调用负例；逐例核对物理 class 验证、Jarde 局部拒绝、回退类 `javac --release 8` 与可重放脚本。五例均通过 Java 8 回退编译；本类调用例保留自类 `<init>` `Methodref`，并以 `void create(Integer)` 避开无关的非 void 返回恢复缺陷。

## 2. 同轮构造器候选与投影

- [x] 2.1 在现有 class-source 同轮恢复接缝证明完整 AST/SSA 的无参 `Object()` 调用加 `return`、无异常表/隐含效果，并交付已证参数槽/名；以正例两种 debug 变体和正文使用/委托负例的定向 Rust 测试验证。
- [x] 2.2 复用 reader 方法 Signature 解析/擦除及结构化类型拼写，只在顶层 Object 直接子类、无同类 `<init>` 绑定与注解冲突时原子写出构造器类型参数和泛型参数；以完整类及合法/错误调用方 Java 8 重编、反射和运行验证。
- [x] 2.3 覆盖来源、局部拒绝、essential/all 正文、预算/取消及相邻有正文泛型方法不变；运行定向 Rust 测试、格式与 `git diff --check`，记录仍拒绝的扩围形状。

## 3. 独立验收

- [x] 3.1 root 审读正文候选的 AST/SSA 完整性、签名作用域、构造器调用绑定和原子发布；独立重放正反例，运行 reader/query、类源码/相邻泛型回归、适当 Clippy、`cargo fmt --all -- --check` 和 `openspec validate recover-proved-generic-constructor-signatures --strict`，记录于 [verification-root.md](verification-root.md) 并清理私有 Cargo target。

### 本片保留的拒绝范围

只投影带有效方法 `Signature`、局部类型变量与擦除完全匹配、物理及泛型 `throws` 皆空的构造器；正文必须精确为 `super(); return;`，且对应 Code/SSA 只有 `aload_0; invokespecial Object.<init>()V; return`，无异常表和额外效果。本类含任意 `<init>` `Methodref`、参数被读取或写入、`this(...)` 链、正文 fallback、嵌套类、非 Object 父类、接口、参数/type-use 注解、varargs 或预算/取消未完成时均保留物理声明并局部拒绝。更宽父类/委托/正文与异常形状仍需逐项证明后再扩围。
