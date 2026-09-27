# DT-27 泛型方法引用首片：root 独立验收

固定输入为 `TypedRefs.class`，SHA-256 `be4ad8a5dffaa846d5ddc99624670f6c8085ab9bff1f34e58f663f40b6cc0334`；固定 JADX HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。实现分支提交 `a54499401f2037e206447fc4244f3ff3fec9ab8c` 已在主线独立审阅。验收仅针对这份 Java 8 完整类的三个直接返回站点及文档列出的近邻，不把 DT-27 整单元标为追平。

`TypedFunctionalTarget` 先由精确方法 `Signature`、擦除、类头与直接 Code 形状选出候选，`lambda::plan` 分别核 erased→instantiated 与 instantiated→implementation 转换，绑定形态仅允许入口 local 0 的 `this`。`generic_return_candidate` 再逐一核 Code、单条 Program Return、SSA 值流、唯一 bootstrap 站点、BCI/CP、捕获与异常边，class-source 在同一预算事务内发布声明和正文。回退依据显式 `generic_signature_projected` 状态，不从 marker 或源码字符串猜测事务是否成功。审阅未发现与该有界形状冲突的路径。

root 在该提交上用新的 `/private/tmp/jarde-dt27-root-target` 构建 CLI，运行 `python3 openspec/evidence/java-syntax-2026-09-28/dt27-typed-functional/replay.py /private/tmp/jarde-dt27-root-target/debug/jarde-cli /private/tmp/jarde-dt27-root-replay`。重编原源码得到的 class 与冻结文件逐字节相同；原 class、固定 JADX、Jarde 的完整源码都通过 `javac --release 8` 和 `java -Xverify:all`，运行输出 `5:5:xy`，三个反射泛型返回类型依次为 `Function<String,Integer>`、`Function<String,Integer>`、`Supplier<String>`。Jarde 保留 `Integer::parseInt`（BCI 0/CP #13）、`this::length`（BCI 1/CP #17）、`this::label`（BCI 1/CP #20）。

同一回放核过伪造参数/结果 `Signature`、不兼容 instantiated SAM、额外效果与 handler、可空或求值型绑定接收者、holder 替换：这些 class 可验证，但泛型投影拒绝；输出预算停止不写半份源码。root 的 `cargo test -p jarde --test class_source --locked` 为 87/87，通过取消与来源断言；`openspec validate recover-typed-functional-method-references --strict` 和 `git diff main...HEAD --check` 通过。实施代理另完成 `jarde-java` 全测试及 workspace check。完整 `jarde --tests --features test-support` 中 `p3_new_value::a_leftover_two_instructions_read_keeps_its_refusal` 的拒绝文案断言失败，代理在未修改的 `origin/main` 复现了相同结果；它是独立测试债务，不纳入本项代码修改。
