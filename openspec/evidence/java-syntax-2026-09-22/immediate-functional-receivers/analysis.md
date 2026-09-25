# 即时函数式调用接收者

入口：`python3 replay.py --cli /path/to/jarde-cli`。脚本从本目录三组 Java 8 原源码重新编译，强制核对冻结 class SHA，再用 `java -Xverify:all` 执行原 class，运行 JADX 1.5.6 和 Jarde 完整类恢复、原样重编并执行可编译输出。JADX 给无包名 class 加 `defpackage`，脚本只给 source-only runner 加同一包声明，不编辑 JADX/Jarde 的 subject。root 使用 CLI SHA-256 `8b86c7293387e4be385630884109cb0effa4d044762788ac019abb7cbdd756e9` 在本目录和 `/tmp/jarde-immediate-functional-root-replay-20260923` 独立重放，两份 `summary.json` 字节相同。

| 输入 | 冻结 class / Code | 原 class 与 JADX 整类 | Jarde 整类 |
| --- | --- | --- | --- |
| `direct-array`：`((IntFunction<int[]>) int[]::new).apply(n)` | 1039 B / 3；SHA `0c4544d1a9334c35d5a5191ba4ef2d17200a8ce9f0ca34b2bd6331538310ea98` | 均编译；`0`、`3`、`NegativeArraySizeException` 逐行相同 | 零引用；`((int p0) -> DirectArrayCtorRef.lambda$make$0(p0)).apply(n)` 被 javac 报“此处不应为 lambda 表达式” |
| `direct-method`：`((IntUnaryOperator) Math::abs).applyAsInt(n)` | 943 B / 2；SHA `de0b0e15bdb199295ce055f36e46cb7dcd56854a1738832ccd8931f5a5053dc3` | 均编译；`0`、`3`、`7` 逐行相同 | 零引用；`java.lang.Math::abs.applyAsInt(n)` 被 javac 报“此处不应为方法引用” |
| `bound-control`：先存 `IntFunction`/`IntUnaryOperator` 再调用 | 1495 B / 4；SHA `aa517756621a289f1a5cf9967a35b6c1e9f42e58e1d26bac74d06973073fd4b7` | 均编译；三行输出与原始类一致 | 零引用、可编译，三行输出与原始类一致 |

`results/<case>/` 保留完整 Java 源、原/JADX/Jarde 的 javac 日志、执行输出与 `javap`。两份失败 class 均无引用，说明当前恢复成功标记不能代替 Java 语法合法性证明；局部对照排除了数组分配或接口调用本身未恢复的解释。

三份原 subject 又单独放入 `tests/fixtures/p3-immediate-functional-receivers/v8/`；root 逐字节重编三份 class 并用冻结 class 与重编 class 各自执行 runner，六次均与证据输出相同。reader census 从 `(151,1086,98,346,8)` 更新为实测 `(154,1095,98,346,8)` 并通过，corpus fingerprint 从 368 个输入增加这 10 个非 Markdown 文件至 378 个，重生成后五项验证通过。

`mechanism-only/` 是**手工方案实验，不是 Jarde 输出**：只给裸露接收者补 `(IntFunction)` 或 `(IntUnaryOperator)` 目标类型，两份完整控制源码均由 Java 8 编译，且 `-Xverify:all` 执行与原 class 逐行相同。这支持复用现有 `Cast` 与工厂 `presented` 类型，而不是引入“数组构造器专用节点”或编译时凭文本补括号。工厂目标与调用 owner 不同或缺失时，必须拒绝；不应借此改变函数体内的 SAM 动态参数适配。更广的显式 lambda 探针另暴露 javac 合成 `lambda$` 方法名冲突，已有 `synthetic-names/` 债务承接，不混入当前改动。

对应 OpenSpec：`../../../changes/type-immediate-functional-receivers/`。

## 实施后独立重放

root 以当前工作树构建 `jarde-cli`（SHA-256 `6e2fce612d014ed327c9891981805720a1d5d276e68daeb3be43f8f33ff2dd9a`），将同一脚本分别输出到 `post-fix/` 和独立的 `/tmp/jarde-immediate-functional-postfix-root-20260923/`，两份 `summary.json` 字节相同。三份冻结 class 的 SHA、Code 数、原类与 JADX 输出均未变。修后 Jarde 的三份完整类均零引用、`javac --release 8` 成功、`java -Xverify:all` 成功，运行输出与各自原 class、JADX 完全相同。`direct-array` 生成 `((java.util.function.IntFunction) ((int p0) -> DirectArrayCtorRef.lambda$make$0(p0))).apply(n)`；`direct-method` 生成 `((java.util.function.IntUnaryOperator) java.lang.Math::abs).applyAsInt(n)`。先存局部的对照没有新增接收者 cast。`post-fix/<case>/` 保留未经手改的 Jarde/JADX 完整源码与编译、运行结果；已修好的两份不再生成 `mechanism-only/`。
