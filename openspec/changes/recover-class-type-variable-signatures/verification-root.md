# 类级类型变量恢复：root 独立验收

## 结论与实现边界

已在现有 reader `Signature` 树、类源码声明与方法投影接缝闭合顶层普通类/接口的类变量子集。类 `Signature` 的变量、首边界擦除及父类/接口数量和顺序先与物理头逐位置核对；只有完整类头发布后，方法擦除与源码拼写才接收这份类变量作用域。类候选失败时保留物理头，方法不会孤立写出未声明的变量。带泛型实参的父类/接口需证明继承成员兼容性，本片拒绝；没有模仿 JADX 的 `fixTypeParamDeclarations` 从使用处补造变量。

源码层重建完整类头，不在已拼好的字符串里替换类型名。原 class `Signature` 的原始字节与拒绝原因保留在 `ClassSourceDeclaration`，源码文本也写出相应注记。带 Code 的直接参数返回沿用同轮 AST/SSA 候选；无 Code 的方法沿用物理抽象/本地声明路径。方法自有类型变量、annotation、varargs、`Exceptions` 和同类 Methodref 的现有门没有被绕过。没有新增 crate 或全局泛型求解器。

## 三方执行与负例

root 重新执行了 [`evidence/replay.sh`](evidence/replay.sh)，退出码为 0；脚本私有 Cargo target 在退出时清理。完整输入、哈希、`javap`、JADX/Jarde 声明与变造方式见 [`evidence/class-type-variable-boundaries.md`](evidence/class-type-variable-boundaries.md)。`ClassVariableBoundary<U>`、`U extends Number & Comparable<U>` 以及 `U extends CharSequence` 的无正文接口方法，在 `-g`/`-g:none` 下均由 Jarde 完整类 Java 8 重编，并让本轮重新编译的泛型调用方 `-Xverify:all` 运行。值分别为 `class-variable`、`19`、`interface-variable`；原/JADX/Jarde 的 class 变量均为 `[U]`，已发布方法的泛型参数和返回均为 `U`。脚本会在任何正例调用方编译失败时退出，运行 classpath 不含预编译 runner 作为后备。

三项只改 `Signature` 的 verifier-valid 负例分别把 class 父类从物理 `Object` 谎报为 `Thread`、把物理接口 `Runnable` 谎报为 `Callable`、把方法所用 `U` 改成未声明的 `V`。Jarde 对前两项拒绝类头、对最后一项保留已证明的类头但拒绝方法头；对应的物理源码均能重编。JADX 1.5.6 则把谎报的父类/接口及未绑定 `V` 写入源码。合法的类/方法同名 `<T>` 仍保守拒绝方法投影；成员内类使用外层 `T` 在扁平单类输入下也拒绝，不把缺失外层作用域猜成内层声明。内类单类源码另有构造器赋值语法失败，未算作本项泛型失败。参数化父接口 `Comparable<U>` 的专门 Rust 反例确认整组继承成员未证时不发布 `<U>`。

## 回归与门槛

- `cargo test -p jarde --test ordinary_generic_projection --locked`：14/14；其中完整泛型调用方/反射、无正文接口、参数化父接口拒绝、essential/all 和低预算原子性为本项覆盖。
- `cargo test -p jarde --test generic_method_projection --locked`：9/9；原本将合法 `ClassVariableProbe<T>` 当拒绝的旧断言已按新能力改为正例。
- `cargo test -p jarde --test generic_method_budget --test class_source --locked`：4/4、47/47。
- reader 的 `signature` 单测 12/12；query 库单测 7/7。reader 全库运行 170/171，唯一失败是共享工作树新夹具使既有总量断言得到 `(196, 1260, 124, 562, 8)`，旧常量仍为 `(164, 1152, 98, 381, 8)`；它不是本项擦除证明的失败，留给独立的夹具清单集成门槛。
- `cargo clippy -p jarde -p jarde-reader --lib --locked -- -D warnings` 加既有五项允许项通过；`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-class-type-variable-signatures --strict` 通过。

root 的 1.3 GiB 私有测试 target 与一次意外重建的 877 MiB 共享 `target/` 均已清理；agent 的私有 target 由重放脚本清理。字段 `Signature`、带泛型实参的继承头、内类外层变量、复杂方法正文与外部依赖闭包仍是分开的后续范围。缺失的嵌套泛型引用类已经由[独立对照](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/nested-missing-signature-reference.md)归为编译 classpath 边界，不借本项造类名或自动下载依赖。
