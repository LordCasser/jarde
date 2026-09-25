# 验收记录（2026-09-23）

修后 CLI SHA-256 为 `6e2fce612d014ed327c9891981805720a1d5d276e68daeb3be43f8f33ff2dd9a`。用 `../../evidence/java-syntax-2026-09-22/immediate-functional-receivers/replay.py` 重新编译三份冻结 Java 8 输入，并分别写入证据目录的 `post-fix/` 与 `/tmp/jarde-immediate-functional-postfix-root-20260923/`；两份 `summary.json` 字节一致。原 class、JADX 和未手改的 Jarde 完整类对三组 0、正数、负数输入均由 `javac --release 8` 编译并在 `java -Xverify:all` 下执行，输出逐行相同。两份即时调用均零引用，先存局部对照未新增接收者 cast。完整源码、编译与运行输出见 `post-fix/<case>/`。

独立复查 `build::call_expr`：只有直接 Lambda/MethodReference 接收者会进入新分支；factory 的 `presented` 必须是可拼写的引用类型，调用必须是 `InterfaceMethodref`，且其 owner 与 factory 类型完全一致。随后复用 `cast_argument`，表达式只求值一次，工厂 BCI 仍是主要来源、调用 BCI 作为 derived 来源。缺失、不同型、数组型与不支持的 owner 走已有拒绝路径；普通局部接收者正文不变。未新增 AST 节点、Emitter 分支或第二次恢复。

验证：`p3_immediate_functional_receivers` 2/2、Builder 定向单测 2/2、`p3_array_initializers` 4/4、`p3_invocation_arguments` 3/3 且被忽略的 JDK 执行项单独通过、reader class census 通过、corpus fingerprint 5/5（另 1 项仅供显式重生成）、`cargo fmt --all -- --check` 通过、`openspec validate type-immediate-functional-receivers --strict` 通过。

`p3_lambda_adaptation` 的文本断言及被忽略的完整类 JDK 项仍失败。修复前冻结 CLI 与修后 CLI 对失败方法 `strings()` 均输出 `LambdaAdaptationSupport::pick`；完整类仍在数组、无捕获实例与构造器适配处 javac 失败。这是已单列的 `preserve-lambda-descriptor-adaptation` 未完成范围，不是此次即时接收者 Cast 的回归。严格 Clippy 的全工作树门禁尚受其它在途改动阻断，不把该门禁写成通过。
