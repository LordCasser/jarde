# root 独立验收：Java 8 平台集合 owner

## 结论与实现审读

`List`、`Collection` 的精确 `invokeinterface iterator()Ljava/util/Iterator;` owner 现可在接收者已呈现源类型分别为 `java.util.List`、`java.util.Collection` 时进入原有 `iterable_for_each_candidate`。root 审读确认新增门槛只改变初始调用 owner/接收者类型的准入；`Iterator.hasNext/next` 的精确符号匹配、SSA 独占消费、首动作/原位 cast、逐调用 handler 集合、来源、预算及原子提交均继续复用 direct `Iterable` 的已验收证明。新 owner 只接受完整限定名，避免把同包自定义短名当成平台类型；用户子接口和非 `Iterable` 同名方法保留普通 `while`。

## 冻结三方重放

root 在独立 `/tmp/jarde-platform-root-target` 执行 `cargo build -p jarde-cli --locked`；最终格式修正后 CLI SHA-256 为 `de3c25b9e0b0b8a533fc501071e90117db0d897f71cccfdeb1e658b3723a1031`。对[接口 owner 原始探针](../../evidence/java-syntax-2026-09-24/iterable-subtype-owners/analysis.md)的 debug/no-debug `SubtypeOwners.class` 分别执行 `class-source --policy single-class --release 8`，再把原 class 或 Jarde 完整类与同一 `TextIterable`/runner 以 `javac --release 8` 重编并 `java -Xverify:all` 运行。两版均为 `list=6, collection=6, textIterable=6`；List/Collection 各一个增强 `for`，自定义 `TextIterable` 保留 `while`。Jarde 两版源码 SHA-256 分别为 `25dc14b5a63577a119d9359ca8ec10c5e38138dc9b8dbef239f00e6c7c9aa754` 与 `a58119108b4ac5f378c34cf54978ae1129b65e22103d0711c68cfeada4364cbd`。既有 JADX `-g` 对前两者输出增强 `for`，`-g:none` 退回 while；Jarde 两版都输出合法增强 `for`。

root 另从[边界 fixture](../../../tests/fixtures/p3-iterable-foreach/PlatformIterableOwners.java)独立编译 `-g`/`-g:none` 原 class，其最终 SHA-256 为 `8d7cecac0f89abd0b44b252b1d7027570b2ee1b35f7745a6def33a5e522519d7`、`6b5c10afdccd87773eaa44e18fc3b4c7845b7a26e170b053f8b5c3a010f4a109`。用同一 CLI 生成的两份 Jarde 完整类 SHA-256 为 `abcdf39927dfa79e11a8888678f80b80ef7b27fc41669cceb5cac1b7a6c8e55c`、`8eea86eddc82f54d52a711d2aba68aa203d3f9a0a8a01d6bf848af032c7677cc`。原/Jarde 均以 Java 8 重编、验证并执行，每版 11 行逐字相同，包括 `continue` 的 `skip=2`、空/null、坏元素 `ClassCastException` 及 cast 后 `touches=1`。`listCast`、`collectionCast`、`listSkipEmpty` 三个方法写增强 `for`，额外 `next`、iterator 逃逸、自定义子接口与同名非 `Iterable` 均为 `while`。root 将冻结 JADX 两版完整源各自重编、验证，11 行均与对应原 class 相同；这份 raw fixture 中 JADX 两版都保留显式 while。完整 class/runner/JADX 产物和调用 BCI 见[三方边界证据](../../evidence/java-syntax-2026-09-24/iterable-subtype-owners/boundaries/analysis.md)。

异常负例独立从 `PlatformIterableHandlerBoundary` 编译，原 class SHA-256 `0843647c94ddcb1a18eec100ad2556b1c713ec359af9c6f6cd3cf883dcb6748d`、`java -Xverify:all` 输出 `1`。Jarde 没有增强 `for`，但该保护区当前 explanation-only，完整类不能重编；它只验收拒绝边界，不能声称 Jarde 运行等价。该结构债务由既有 [异常局部作用域任务 2.5](../preserve-local-scope-across-exception-regions/tasks.md)追踪。

## 回归与磁盘

root 独立执行新增及原有 `p3_iterable_foreach` 5/5、相邻 `p3_array_foreach` 4/4、`p3_wrapped_array_foreach` 4/4、`p3_loop_transfers` 5/5，以及 `cargo test -p jarde-java --lib --locked` 140/140。新测试另覆盖 `listCast` 的调用 BCI 1/10/19 来源、essential/all 正文一致、低预算与预取消不发布半折叠循环。`cargo fmt --all -- --check`、`openspec validate project-proved-platform-iterable-owners --strict` 与 `git diff --check` 在最终实现下均通过；root 第一次格式检查发现的 `build.rs` 换行差异已由实施代理修正。代理的 `/tmp/jarde-platform-iterable-target` 经 `cargo clean` 删除 1.4 GiB 构建文件；root 的第一个独立 target 删除 7173 文件/2.7 GiB，最终格式修正后重建 target 又删除 2702 文件/1.0 GiB。仓内无 Cargo target，可用磁盘约 17 GiB。
