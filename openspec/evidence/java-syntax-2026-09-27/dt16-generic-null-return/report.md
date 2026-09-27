# DT-16：实例泛型方法直接返回 `null`

固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`TestUsageInGenerics` 的有效源码断言是 `public <T extends A> T test() {`，其方法正文 `return null;`；本例将界替换为 JDK `Number`、类型移到顶级类，以隔离 DT-16 的方法形参/返回语法，不把成员类路径 DT-02/03 混进失败点。`TestGenericsInArgs` 另涉及 `List<? super T>`、`Set<T>` 与较复杂正文，留在 DT-16/17/18 的后续组合验收。

[replay.py](replay.py) 从 [NullResult.java](NullResult.java) 和外部 API consumer [Runner.java](Runner.java) 以 `javac --release 8 -g:none` 编译。它从固定 JADX checkout 的 CLI 和 Jarde `class-source` 取得完整 `NullResult`，分别与同一个 `Runner` 重编，并运行 `java -Xverify:all`。命令是 `python3 replay.py baseline /absolute/path/to/jarde-cli`；修后 `fixed` 模式写入独立 `fixed/` 并要求三方均成功。冻结源码、原 class hash、`javap -c -p -s` 的精确 `aconst_null; areturn` 字节码、实际两份反编译源码、诊断和结果均在 `outputs/`。

原始 class 和 JADX 完整源码都通过 Java 8 重编及验证运行，输出 `true:T:java.lang.Number:T`，同时检查带显式 `<Integer>` 的调用结果、反射类型变量名称/界和泛型返回类型。Jarde 的方法体正确显示 `return null;`，但 `ClassSourceMethod::project_method_signature` 将泛型头拒绝为 `generic_source_shape_unproved`，当前源码声明是 `public java.lang.Number value()`；原 consumer 因 `Number cannot be converted to Integer` 无法重编，故未运行。不能把 Jarde 的方法体可读性计为本单元成功。

reader 已有方法 `Signature` 解析与 descriptor 擦除证明；`jarde-java::generic_return_candidate` 只产出空 `return;`、直接参数返回、条件参数返回和两个成员创建形状，单语句 `return null;` 没有同轮候选。`class_source::generic_method_declaration` 也只接受静态参数来源的 `T` 返回。缺口可在既有候选接缝增加严格的无参数、无效果、唯一 `aconst_null; areturn` 实例方法形状，并在现有方法签名投影里证明其可以作为任意已解析引用类型变量的值；不需要第二个 Signature parser 或通用类型推断机制。具体拒绝边界在 [独立 OpenSpec](../../../changes/recover-proved-generic-null-return/) 中约束。JADX 的泛型方法输出用于定位并比较，不替代 Jarde 的同轮正文证明。

## Fixed 验收

`python3 replay.py fixed <absolute-path-to-jarde-cli>` 已通过。它将原始 class、固定 JADX 源码和 Jarde 完整源码分别与同一个外部 consumer 按 Java 8 重编，均以 `-Xverify:all` 运行，并逐字得到 `true:T:java.lang.Number:T`；三方的反射类型变量、界和泛型返回一致。固定输出在 [fixed/](fixed/)；冻结 baseline 保留在 `outputs/`。

新增回归覆盖实际编译的 `public <T extends Number> T value() { return null; }`、精确物理 descriptor/方法记录，以及副作用、异常 handler、条件合流、类级变量、类型注解、不同界/descriptor、同类重载调用和静态方法拒绝。另验证 Signature 擦除不一致、输出预算停止和取消不发布半个泛型头；完整独立查询仍保留 `()Ljava/lang/Number;` 物理方法。

本 worktree 的完整 `cargo test -p jarde-reader` 有一项既有 corpus census 断言失败：实际是 351 类、1,804 个方法体、160 个 handler、1,004 个控制流目标和 8 个 subroutine，断言记录仍是 330、1,730、160、1,000、8；其余 176 项通过。相关 `signature::tests` 及 `method_signature_proof` 通过。完整 `cargo test -p jarde-java` 的集成测试编译被既有 `anonymous_allocation_candidates.rs` tuple 解构错误阻断（函数返回 3 项、测试解构 2 项）；`cargo test -p jarde-java --lib` 的 219 项通过。该两处限制不在 DT-16 范围。

Root 在合入 `de39fbd4` 主线后独立复跑 `fixed`：三方完整源码与同一 consumer 的 Java 8 重编、`-Xverify:all` 和反射结果一致；`generic_method_projection` 13 项、`jarde-java --lib` 219 项、workspace check、fmt 与 147 项 OpenSpec strict 均通过。物理方法仍是 `()Ljava/lang/Number;`，DT-18 的 `List<String>` null 返回另有独立差距与规格，未被本变更误投影。
