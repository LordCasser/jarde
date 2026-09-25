# 独立验收记录

## 基线（任务 1.1–1.2）

root 从[冻结的探针源码](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/ArrayWrappedBinding.java)独立执行 `javac --release 8 -g:none -Xlint:-options`，所得 `ArrayWrappedBinding.class` 与冻结 class 的 SHA-256 均为 `0f024e6ebb40b16a0da82f2058374f715e233b13457bdb952abc236767d1d9ca`。主体源码 SHA-256 `c3ae78d1a80a205fe9cde417568e3ee54230493b48debdbd3db2be2924933d38`，runner 为 `9907e757c9fd6f4e91e392707e1695854c1cb7b3005e5a5b43159c3be2ce51f0`。

root 将原源码、[JADX 完整类](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/ArrayWrappedBinding.jadx.java)及[当前 Jarde 完整类](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/ArrayWrappedBinding.jarde.java)分别同 runner 重编，三者 `javac --release 8 -g:none -Xlint:-options` 均退出 0，`java -Xverify:all` 均退出 0。原/Jarde 的 13 行逐字相同；JADX 的其余 10 行相同，但 `plusTickFirst` 为原 `92`、JADX `4`，`callMutateThenArray` 为 `1091`、`1003`，`readAfterEffect` 为 `91`、`3`。原 class 和 Jarde 在这些运行里读取修改后的 `[91]` 元素，JADX 的增强 `for` 在修改前缓存旧值 `3`。

基线语法核对：JADX 对六个候选写增强 `for`，对同轮两次读取的 `twoReadsWithMutation` 保留计数 `for`；当前 Jarde 七个方法均为可执行计数 `for`。`array[i] + tick()` 和 `sum += array[i] + tick()` 是先读后调用的正例；`mutateAndTick() + array[i]`、`consume(mutateAndTick(), array[i])`、先改数组再读及同轮两次读取是拒绝例。null 与抛错路径已在同一 13 行轨迹中核对；这组输入的 null 在长度读取时即抛出，不能单独证明任意元素读取位置均可移动。

复核使用独立目录 `/tmp/jarde-wrapped-root-verify/{original,jadx,jarde}`，三组命令分别以原、JADX、Jarde 完整类加对应 runner 执行上述编译与 JVM 命令。没有新建 Cargo target；冻结产物、工具版本及更详细的复现命令见[三方证据](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/analysis.md)。

## 实施后 root 独立验收（任务 3.1–3.2）

root 在独立 `/tmp/jarde-wrapped-root-target` 运行 `cargo build -p jarde-cli --locked`，CLI SHA-256 为 `f3f3602b7de0c6123b4ab07c5a71ebf3ced90bdeb77d4fb8bab6f07d7d5f2e7f`。用它对同一冻结 class 执行 `class-source --policy single-class --release 8 --format text`，所得完整类 SHA-256 `2deb4fd1f66d9b0768b4c32a027ea308d93c9ab6f6d6eee22050a0ac7c00527f`. `plusArrayFirst` 和 `wrappedOnlyRead` 分别输出 `for (int arrayElement19 : local2)`、`for (int arrayElement20 : local2)`；其余五个候选仍是计数 `for`，包括三条已证实会被 JADX 提前读取而改变值的路径。

root 将新 Jarde 类与冻结 runner 以 `javac --release 8 -g:none -Xlint:-options` 重编，`java -Xverify:all` 执行；原 class 和新 Jarde 都是 13 行。仅将 JVM helpful-NPE 中 `<local数字>` 归一后逐行 `diff` 无差异；三条关键值保持 `92`、`1091`、`91`，数组最终状态、调用次数和异常类型/传播相同。原/JADX/Jarde 的基线三方重编见上节；JADX 的三条错误值仍是 `4`、`1003`、`3`。本次正例保留元素读取之后的 `tick()`，没有重复求值。

root 独立运行 `cargo test --no-default-features --test p3_wrapped_array_foreach --locked`：4/4；相邻 `p3_array_foreach`、`p3_loop_transfers`、`p3_for_add_store`、`p3_array_access`：23/23；`cargo test -p jarde-java --lib --locked`：140/140；`cargo fmt --all -- --check`、本变更 `openspec validate --strict` 与受影响文件 `git diff --check` 均通过。实施代理已覆盖低预算、取消、真实 BCI 与 essential/all 正文相同；root 复核了候选的首条体语句、加法前缀、唯一数组读取、SSA 索引/数组身份、处理器比较和原子提交点。全量 `cargo test -p jarde-java` 在并发 facade 签名改动留下的 `class_initializer_candidates.rs` 七处、`p3_patterns.rs` 一处旧调用处编译失败，未将它算作本语法子切片失败或擅自混入修复。

文本质量的独立后续债务：正例仍有失去用途的顶部 `int local3; int local4;`，虽不影响重编和执行，但应按[路线图](../../roadmap.md)另做投影后局部用途证明与清理。不能仅按局部名删除。root 独立 target 经 `cargo clean --target-dir /tmp/jarde-wrapped-root-target` 清理 7235 文件/2.7 GiB；可用磁盘由约 14 GiB 恢复到约 16 GiB，实施代理的 target 也已清理。
