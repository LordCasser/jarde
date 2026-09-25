# Root 验收：同槽局部的已证生命周期

## 结论与范围

`ReuseAfterForEach.sumThenReuse([I)I` 在槽 2 先放增强 `for` 的 `int[]` 别名，循环退出后又放 `int`。冻结的预修复 Jarde 在 `-g`、`-g:none` 两种 class 上都把它写成一个数组局部，完整类各有三处 Java 8 类型错误。当前实现沿用 `reuse::Plan`、SSA、canonical CFG、`NameTable` 和现有声明规划，为已证的 Ref/Int 值链分配两个局部；无 LVT 时生成两个稳定名称，有 LVT 时只将 `first` 贴到后段。两份完整类都通过 `javac --release 8` 和 `java -Xverify:all`，runner 与原 class、JADX 一样输出 `22 / 4 / NullPointerException`。[冻结输入与逐 BCI 证明](evidence/analysis.md)及[可重放脚本](evidence/replay.sh)记录三方文本、class 哈希、verifier、SSA、Region 与 CLI 运行。

这不是一般的源码局部重建。新推断只接受 `Ref` 与 `Int`、完整且彼此独立的写入/读取代表值、可归约的平凡 phi、无异常或 call-context 边，以及 canonical CFG 上后段到前段访问块不可达。由此接受前段自身的循环回边与互斥分支；后段能沿回边返回前段时不按 BCI 切开。未分段且确定发生引用/原始类型冲突的声明带两个写入 BCI 拒绝，未实现引用间子类型或原始类型转换的通用赋值判断。调试表是名称和既有范围证据，不代替值/控制流证明。

## 独立重编与边界

root 使用本轮重新构建的 CLI 独立编译和运行 `ReuseAfterForEach` 两种 class，随后 evidence 子代理以另一私有 target 复放同样结果；当前 Jarde 完整文本 SHA-256 分别为 `09bae31b07e4cb2b6063fff37af4ab59544a763c1e6090f42891d407bf37cd6d` 与 `be44c108fa06790ab99850c193e49d5d0b9422c1df0042f2051f18bb4db29416`。essential/all 的 Java 正文相同，方法源图仍覆盖数组别名及后继写入的 BCI 3、4、16、36、37、40、42、44。

| 边界方法 | `-g` 的 Jarde | `-g:none` 的 Jarde | 验收含义 |
| --- | --- | --- | --- |
| `sameType` | 方法隔离重编、运行等价 | 方法隔离重编、运行等价 | 同为 `int` 不因值链断开而强制新分段 |
| `exclusiveBranch` | 方法隔离重编、运行等价 | 方法隔离重编、运行等价 | 两个互斥分支各有完整值链，CFG 无后段返回前段 |
| `loopBodyReuse` | 既有双 LVT 范围路径重编、运行等价 | 冲突拒绝；残余文本虽可编译，但输出 `0` 而非 `8` | 后段通过循环回边返回前段，不能将拒绝文本算成恢复成功 |
| `handlerReuse` | explanation-only，缺返回语句 | 同左 | 既有受保护区域/局部生存期缺口；不属本次分段回归 |
| `category2Adjacent` | 既有 LVT 路径重编、运行等价 | 槽 0 的引用/`long` 冲突拒绝，缺返回语句 | `long` 占槽 0、1；不得将槽 1 当作独立局部 |

边界 fixture 原 class 在两种模式均通过 verifier，runner 输出 `11 / 7 / 9 / 8 / 3 / 8 / 4294967300`。Jarde 整个边界类含 explanation-only 方法，不能宣称可完整重编；上表是逐方法隔离的编译和执行对照，[重放日志](evidence/logs/replay.stdout)包含 javac 返回码与输出。Java 9 `Held.use` 的 handler slot 2 已查 SSA 和异常边；独立关闭新分段与冲突门时仍保持原有 explanation-only 结果，故不归因于本次改动。

## 代码与检查

- `reuse.rs` 在既有计划中完成有界 SSA/CFG 分段，逐访问计费和轮询，整份计划成功后才交给后续阶段；`names.rs` 将分段名称表达为逐段可选原名；`build.rs` 对确定的引用/原始类型冲突拒绝同一声明，handler 的 caught 写入由原有头部规则处理。
- `tests/p3_array_foreach.rs` 的新回归从同一源码分别编译 `-g`/`-g:none`，核对类级文本、完整 Java 8 重编、`-Xverify:all` 值与 null 异常、essential/all 正文及源 BCI。数组增强 `for`、本地作用域、参数槽、声明、Iterable、异常作用域、循环转移和资源处理的定向测试通过；`jarde-java` 库单元测试 140/140 通过。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate separate-reused-local-lifetimes --strict` 通过。`cargo clippy -p jarde-java --lib -D warnings` 在保留仓库既有五项 lint 宽免后通过；这些宽免不是本次新增警告。

整仓 `p3_try_local` 两个既有测试仍失败：root 分别停用新 typed split 和新冲突门复验，结果未变，原因是受保护区域的 quoted fallback 跨局部作用域；按[异常区单独任务](../preserve-local-scope-across-exception-regions/tasks.md)处理。其他待处理债务是引用间/原始类型间的完整赋值相容性与泛型 `Signature` 嵌套缺失类的解析，均不混入此最小闭环。root 与证据子代理的私有 Cargo targets 已清理；`df` 显示本轮清理后可用空间约 15 GiB。
