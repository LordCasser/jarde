# 实施与验证记录

本记录覆盖任务 1.2、2.1、2.2。3.x 的独立 CLI 重建、root 审读与 strict 验收由 root 单独执行。

## 实现

`crates/jarde-java/src/build.rs::iterable_for_each_candidate` 仍用同一 SSA、调用消费者、首动作、cast 原位、handler 集合、来源、预算及原子提交证明。候选准入把精确 `iterator()Ljava/util/Iterator;` owner 扩到 `java/util/List` 和 `java/util/Collection`，并分别要求 receiver 的 presented source type 精确是 `java.util.List` 或 `java.util.Collection`。新增 owner 不接受短名，以免与同包自定义类型混淆。`java/lang/Iterable` 的既有 full/simple 源类型路径保持不变。

## 冻结基线与执行结果

边界输入、工具版本、owner/BCI、class/source SHA-256、原与 JADX 重编运行输出见 [`boundaries/analysis.md`](../../evidence/java-syntax-2026-09-24/iterable-subtype-owners/boundaries/analysis.md)。List/Collection 正例、raw `Object` 绑定、循环体原位 cast、`continue`、空/null、坏元素、副作用、额外 `next()`、iterator 逃逸、自定义子接口和同名非 Iterable 都有可执行 fixture。

handler 不一致例子独立编译运行：iterator 的 `next()` 推进一次后抛出，原 Java 8 class 以 `-Xverify:all` 运行并输出 `1`。Jarde 明确拒绝增强 `for`；该形状当前不能被完整恢复，测试只核验拒绝和 explanation-only 说明，不把它作为 recovered 代码等价运行。它没有影响平台正例的完整类重编。

## Cargo 验证

使用独立目录 `/tmp/jarde-platform-iterable-target` 避免复用共享工作区 Cargo 残留。通过：

- `cargo test --test p3_iterable_foreach`
- `cargo test --test p3_iterable_foreach --test p3_array_foreach --test p3_wrapped_array_foreach`

新增平台 owner 测试覆盖 `-g`/`-g:none`，要求 List 与 Collection 都呈现 `for (java.lang.Object ...)` 并保留 `(java.lang.String)` cast。完整 recovered 类均以 `javac --release 8` 重编，在 `java -Xverify:all` 下执行，stdout 与相同选项编译出的原 class 完全一致。essential 与 all 的正文一致；`listCast` 中 iterator owner、`hasNext`、`next` 三个 BCI（1、10、19）都有来源文本。低分析预算返回预算停止，预取消返回取消结果。相邻数组增强循环与 wrapped-array 循环测试均通过。
List `continue` 正例由 `listSkipEmpty` 覆盖，也通过两种调试信息设置、Java 8 重编与完整类运行对照。

测试期间的独立 Cargo target 曾占用 1.0 GiB；执行 `CARGO_TARGET_DIR=/tmp/jarde-platform-iterable-target cargo clean` 后，目录已不存在，释放的构建文件为 1.4 GiB。工作区默认 target 未清理。
