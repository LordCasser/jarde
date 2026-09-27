# DT-17：单个通配符 `List` 参数的空 `void` 方法

基线是本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`。JADX `TestGenerics` 对 `List<?>`、`List<? extends A>`、`List<? super A>` 有显式文本断言，`TestGenerics3` 对 `? extends byte[]`、`? super int[]`、`? super Integer`、`? super String` 有断言。这里把用户定义的成员类界 `A` 换成顶级可解析的 `Number`，避免 DT-02/03 成员类路径；固定 `?`、`? extends Number`、`? super String` 和两个原测试的数组界。`raw(List)` 是无 `Signature` 的负控制。测试中的 `mthExtendsString` 实际参数是 `? super String`，以源/断言内容为准。

[replay.py](replay.py) 以 `javac --release 8 -g:none` 构造原 class，并分别取固定 JADX 和 Jarde 的完整 `Wildcards` 源码。三份类型源码均与同一个外部 [Runner.java](Runner.java) 重编、用 `java -Xverify:all` 运行；runner 逐方法读取真实反射参数类型，避免“能编译但丢失 Signature”被误判为成功。运行 `python3 replay.py baseline /absolute/path/to/jarde-cli`；修后 `fixed` 模式独立写 `fixed/`。原 class SHA、字节码、双方源码和结果在 `outputs/`。

原始与 JADX 的五个泛型参数分别是 `List<?>`、`List<? extends Number>`、`List<? super String>`、`List<? extends byte[]>`、`List<? super int[]>`；无 Signature 控制仍是原始 `List`。完整 Java 8 编译与验证运行均成功。Jarde 的完整源码也编译和运行成功，但五个方法的参数在反射中全部退化为 `java.util.List`；`raw` 仍正确保留原始类型。这是可观测的 Signature 丢失，而非 javac 失败。

reader 的 `Signature` parser 与 `class_source::spell_ordinary_signature_type_with_member_path` 已经认识三种通配符和数组界。具体断点是 `jarde-java::generic_return_candidate` 的 `EmptyVoid` 候选只允许没有参数；同轮存在一个未使用参数时，`ordinary_parameterized_declaration` 得不到正文候选，五个方法都记录 `ordinary_generic_source_unproved`。可复用现有候选、参数槽名和原子方法投影，为“单个 `List` 通配符参数 + 精确无效果 `return;`”增加证明门；无需新增 wildcard parser 或通用类型推断。其它含通配符的方法体、字段、返回值、成员类界留在 DT-17/18 后续切片。

## DT-17 实施验收

`replay.py fixed` 以 `javac --release 8 -g:none` 编译原始类型、固定 JADX 完整源码和 Jarde 完整源码，三个 classpath 均由同一 `Runner` 执行 `java -Xverify:all`。五种通配符反射参数逐字一致，无 `Signature` 的 `raw` 控制保持 `java.util.List`。固定结果、原/JADX/Jarde 源码和运行输出位于 [fixed](fixed/results.json)。

回放同时编译并验证一个独立负例类：参数读取、参数改写、隐藏调用、异常 handler、多参数、成员类界、type-use 注解和同名方法调用均保留 descriptor 的原始 `List`。负例原始 class 和 Jarde 源码都通过 Java 8 编译及 `-Xverify:all`；反射结果见固定结果文件。另将一个合法 class 的 `any` 方法 Signature 改成与物理 List descriptor 擦除冲突的 `Other`，class 仍由 `-Xverify:all` 加载，Jarde 拒绝该签名并仅为该成员保留物理 `List`，其它四个通配符不受影响。

将 `analysis_steps` 限为 100 的回放以 exit 4 停止：先前完整提交的 `any(List<?>)` 仍完整呈现；未能完成的 `ext` 方法保留物理 `List` 参数，后续成员报告停止。输出保存在 [budgeted-Wildcards.java.txt](fixed/budgeted-Wildcards.java.txt)，脚本逐行拒绝不完整的通配符拼写。命令行没有取消注入入口，因此取消没有在这一 Java 回放中单独触发；证明与签名投影仍通过既有预算/取消同一停止结果路径传播取消。

脚本另将 `any` 的 Code 长度改为零，制造 Code 内部结构不完整的 class 文件。class-source 返回原始 `List` 方法声明和停止/拒绝事实，不从仅有的源码外形推断空正文；该损坏输入不属于 verifier-valid 负例，因此只检查 Jarde 的物理来源保留，不送入 JVM。

本切片只准入顶级普通 `Object` 子类里的静态单 `List` 参数、`?`/`? extends`/`? super` 及简单类或基本类型数组界限。引用类型数组界、成员类型、type-use 注解、方法调用/参数使用、多参数和其它 DT-17 正例仍拒绝投影；完整项目与更广的 DT-18 支持不由此验收推断。

Rust 验证通过 `cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test -p jarde-java --lib`（219 项）、`cargo test -p jarde --lib class_source::tests`（20 项）、`cargo test -p jarde-cli --test class_source_cli`（17 项）、reader Signature 相关测试（17 项）及 `openspec validate recover-proved-wildcard-void-parameters --strict`。两个更宽的旧目标另有仓库基线问题：`cargo test -p jarde-java` 的 integration test 在 `anonymous_allocation_candidates.rs` 将三元组解构为二元组而无法编译；完整 reader 测试中 176 项通过，唯一失败是固定 fixture 数量期望 `(330, 1730, 160, 1000, 8)`、当前 checkout 实测 `(351, 1804, 160, 1004, 8)`。这些文件均不属于本次改动。取消没有单独注入测试；本轮直接预算中断证据如上。
