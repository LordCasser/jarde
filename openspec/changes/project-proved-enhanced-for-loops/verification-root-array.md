# root 独立验收：数组直接元素绑定子切片（3.1–3.2）

2026-09-24 从当前共享工作树独立以 `cargo build -p jarde-cli --target-dir /tmp/jarde-enhanced-for-root-target --locked` 重建 CLI，SHA-256 `87cbea28c7912124f999d63ab67eae0f121049acd016a885ec02030d0805f625`，与实施子任务的构建不同。将已冻结的三组增强 `for` 原源码及 runner 复制到 `/tmp/jarde-enhanced-for-root-replay/`，用旧 [重放脚本](../../evidence/java-syntax-2026-09-22/enhanced-for/root-replay/run_audit.py)的同一编译/执行流程、仅更新 CLI 路径及校验哈希，重新从 Java 8 源编译输入、运行 JADX 1.5.6 与 Jarde，再对三个源码版本分别 `javac --release 8 -g:none`、`java -Xverify:all`。CLI 前后哈希一致。另用同脚本将本变更的 `ArrayForeachRefusal`、`ArrayForeachTransfers` 两组源码/runner 置于 `/tmp/jarde-enhanced-for-root-neg-replay/` 重放。

| 主体（原 class SHA-256） | 原/JADX/Jarde 编译与运行 | Jarde 语法 | 结果 |
| --- | --- | --- | --- |
| `IntArrayForeach` `067bf3c5b231811f97d19e9137c7d4019eb2849f8cce91706285120db7d55b58` | 三者均退出 0，6 行 | 两处 `for (int local5 : local2)` | 值、Supplier 调用数和异常类型一致；null 的 helpful-NPE 局部编号 `<local2>`→`<local5>` |
| `ObjectArrayForeach` `b4c1b0ba7b5b1857f4e5369fb57245cb835ec727f180ec9d81c1a4d6d60841b2` | 三者均退出 0，7 行 | 两处 `for (Object local5 : local2)` | 值、hashCode 次数与抛错顺序一致；null 的 helpful-NPE 局部编号同上 |
| `StringIterableForeach` `81a0dcb4a53a51eb721df72f2b30825b33e8fb4dc6f2390e93167f4012e09f36` | 三者均退出 0，10 行 | 保持两处 `while` | 原/Jarde 逐字一致；JADX 第十行 helpful-NPE 把 null 来源描述为 `Iterator.next()`，迭代计数与异常类型不变 |
| `ArrayForeachRefusal` `19a4f569b98fae469cddc625ab2b11af3d2a4dae5635b4526e56b0c007f4e639` | 三者均退出 0，5 行逐字一致 | 四处计数 `for`、一处 `while`，无增强 `for` | 索引体内/退出后用途、错配数组、额外计算、元素逃逸均未被 Jarde 删除；JADX 对 `effectInBinding` 可安全折叠，见下文 |
| `ArrayForeachTransfers` `51fb9055101b7b130f8e402334ee59056d32d36e18fdef95da6078cf1a98f622` | 三者均退出 0，4 行逐字一致 | 一处增强 `for`、一处 `while` | `continue` 正例保留转移，`break` 边仍走保守普通循环 |

前两组唯一不逐字相同的行是 JVM 23 对重新编译类的 helpful-NPE 临时编号描述；只将这段描述归一到异常类型后，五组原/JADX/Jarde 的全部 6/7/10/5/4 行均相同。完整原始行差异保留在 `/tmp/jarde-enhanced-for-root-replay/{int-array,object-array,string-iterable}/comparison-original-vs-*.txt`；该临时目录不是永久 class 真值，物理原 class SHA 与源码在上述仓库证据/夹具中固定。

独立审读 `array_for_each_candidate`：它位于既有 `ForHeader` 形成的计数 `for` 之后，要求前置长度缓存、零起点、单次加一、同一 SSA 数组值、元素类型、仅有规定用途的索引/长度以及元素局部不逃逸；数组捕获原位保留。长度 header Phi 仅允许入口值与自身回边透传，循环体不可写长度槽，且仅允许测试 load 为真实消费者。异常 handler ordinal 在长度读取、元素读取及测试上必须一致；不满足即保留原形。投影前构造带来源的完整候选，成功后才移除长度缓存/元素绑定，拒绝路径不部分隐藏旧语句。`StmtKind::ForEach` 和 emitter 仅增一条必要语句形态，未新建 IR/pass/依赖。来源测试覆盖四个正例的数组捕获、长度、索引、元素读取/存储与更新 BCI，essential/all 正文一致；预算测试覆盖一个低限点和取消，不声称穷举每个限额。

root 在独立 target 重跑 `cargo test --test p3_array_foreach --no-default-features --locked` 4/4、`cargo test -p jarde-java --lib --locked` 140/140，以及 `p3_array_access`、`p3_loop_transfers`、`p3_for_add_store`、`p3_loop_test_values`、`p3_loop_try_handler_entry` 全部定向通过；`cargo fmt --all -- --check`、`openspec validate project-proved-enhanced-for-loops --strict` 退出 0。全量 `cargo test -p jarde-java --locked` 仍因共享工作树另一个在途修改把 `recover_for_class_source` 改成三参数，而 `class_initializer_candidates.rs` 七处、`p3_patterns.rs` 一处仍用双参数，编译错误 E0061；该错误不在本语法切片内，没有顺手修改这些测试。

验收后运行 `cargo clean --target-dir /tmp/jarde-enhanced-for-root-target`，清除 8177 文件、3.0 GiB，磁盘剩余约 16 GiB；实施子任务的独立 target 也已清理。

**后续语法点。** JADX 在 `effectInBinding`（先 `iaload`，再调用 `tick()`）输出增强 `for`，Jarde 当前保留可执行的计数 `for`。本切片的直接元素绑定证明故意拒绝含额外计算的首条绑定；若扩大，必须证明数组读取在可能有副作用的调用之前，并把读取移到增强 `for` 隐式元素绑定、把后续表达式留在循环体。反向的“先调用再读数组”不能这样移动，否则顺序会变。此形状单列后续任务，不能将本次四个直接绑定正例的验收写成数组遍历全面追平 JADX。
