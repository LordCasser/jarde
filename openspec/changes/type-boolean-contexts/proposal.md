## Why

两种 boolean 上下文被恢复成另一种程序（本次复核独立复现，均为受控 `javac --release 8 -g:none` 样本）：

- 返回 `Z` 的方法 `public static boolean isZero(int x) { if (x == 0) { return true; } return false; }` 的正文出现 `return 1;`/`return 0;`；
- boolean **调用结果**作条件（`if (flag())`）的正文出现 `if (flag() != 0)`。

把这两段正文分别包进方法自己的签名，javac 都拒绝：`int cannot be converted to boolean`、`incomparable types: boolean and int`。也就是说产物在「声称 Java 且无拒绝」的形态下无法编译，调用方拿不到可运行的 Java。

根因位置由 [完成复核](../../completion-review.md) 指出：`crates/jarde-java/src/build.rs` 的 `Return` 分支按值渲染、未按返回 descriptor 定型；同文件的 `condition` 只对 `boolean` **参数**做特判（`boolean_parameter`/`parameter_boolean` 只认「单条 `load` + 该参数槽 descriptor 为 `Z`」），返回 `Z` 的调用结果没有这条证据，于是落回 `!= 0` 的整数比较。两者是同一族最小闭环，不扩展成全局类型系统重写。

## What Changes

- 值处于 boolean 上下文时 MUST 以 boolean 呈现：`Z` 方法的 `return` 写成 `true`/`false`；条件的分支测试在其操作数被证明为 boolean（boolean 参数/局部、返回 descriptor 为 `Z` 的调用结果、由此可证明的值与字面量）时写成真值测试（`if (flag())`），MUST NOT 写成 `1`/`0` 或与 `0` 的整数比较。
- 层不能证明上下文（或该上下文里值）的 boolean 类型时 MUST 拒绝该区域并保留 bytecode 与 origin，MUST NOT 发布另一个编译器拒绝的文本。
- 验收：两个形状的受控 fixture 与修正前后文本；用本次运行自己的事实派生签名、把产物交给 `javac` 编译执行（沿用 `tests/p3_execution_comparison.rs` 的包装器路径，`boolean` 参数/返回已在该文件中受支持）；两侧执行结果逐项相同；变异；正向对照证明真正 `int` 返回的方法仍打印 `1`/`0`、int 比较仍打印 `== 0`。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增「boolean 上下文按 boolean 呈现，证据不足则拒绝」的要求（主规格此前没有覆盖返回/条件定型的要求）。
- `recovery-validation`：新增「boolean 上下文的产物在成员自己的签名下编译并逐项执行对照」的验收要求。

## Impact

实现落在 `crates/jarde-java/src/build.rs`（`Return` 分支的取值路径与 `condition` 的 boolean 特判）；`emit.rs` 的打印与 `ExprKind::Boolean` 字面量已存在，不需要新节点。本 change 与 [group-call-receivers](../group-call-receivers/proposal.md)、[spell-array-types](../spell-array-types/proposal.md) 都改 `crates/jarde-java`，三者 MUST 按 [路线](../../roadmap.md) 顺序**串行**实施（本 change 为第 2 个），彼此不依赖对方代码。

交付包含受控 fixture（源码、class 字节、来源 README、`tests/fixtures/README.md` 登记、fingerprint 再生成、reader fixture census 更新）、精确文本回归、javac 编译与执行对照、变异与正向对照。反例 MUST 在修正前先记录（命令、正文、javac 的拒绝信息）。

非目标：不新增 crate、依赖或 verifier；不建通用类型系统或推断；不使用返回 descriptor、参数槽类型与既有 callee descriptor 之外的证据，也不改 `value_type` 对其它类型的决定；不重开 R8/R9 与已归档的递归界、打印修正；不声称一般语义等价，也不声称覆盖整类类型缺陷；不做性能工作（`optimize-demand-workloads` 保持 0/22）。

当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。
