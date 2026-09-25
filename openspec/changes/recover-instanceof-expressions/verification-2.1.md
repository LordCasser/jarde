## 2.1 独立验收（2026-09-24）

冻结 `InstanceOfProbe.class` SHA-256 为 `d8f4a437d0cd0f3ec12672791f27b1c11cfbcdc6885d10eddcd3114aa0838e56`。root 用实现后的 CLI `/tmp/jarde-instanceof-replay/jarde-cli`（SHA-256 `336fda92b93df11a33299cb015df7b925b4ade22970727a21ff09b4b32156b71`）重放 `class-source --policy single-class --release 8 --evidence source_map`：15 个成员全部恢复，没有 `@bytecode`。`widenedString`、`called` 用 `(java.lang.Object)` 保留合法类型测试，`functional` 先固定 `Runnable` 方法引用目标再加宽；`local` 为 boolean 声明，`parameter`、`branch` 均写出真实类型测试。BCI 1/3/5 类型测试与调用在 source map 中保留直接来源。

root 原样重编永久 Java 源码，class 与冻结输入逐字节相同；原类和 Jarde 完整类均以 Java 8 编译、通过 `java -Xverify:all`，同一 Runner 22 行逐字节相同（SHA-256 `37a6cb0205157884d2e39a7b98f19271dec72a8e025f21b27626d46a843494f8`）。同一冻结类的 JADX 1.5.6 输出仅去掉自动加的 `package defpackage;` 后仍有三处 Java 8 编译错误：两处 `String instanceof Integer` 及一处未赋函数式目标的方法引用。Jarde 这里优于 JADX 的直接拼写。

root 还重编 [静态类型边界](../../evidence/java-syntax-2026-09-22/instanceof/type-boundaries/InstanceOfTypes.java)及其 helper/Runner。Jarde 的 `String[]`→`Object`、`String`→`Object`、相关接口、调用与方法引用均可完整编译；原/Jarde `-Xverify:all` 的 11 行相同（SHA-256 `94a63eb6a5605a3b5fadf736c0e1ca5ea69c70b35cb69ef8cc1be7a608e9e15f`）。另用 `((String) value) instanceof CharSequence` 的 Java 8 探针核对，Jarde 保留原 `checkcast`，对 String/null/Integer 依次输出 `true`、`false`、`java.lang.ClassCastException`。这些补充证明 2.2 的关键边界，但未完成该任务全部相邻回归，故只勾 2.1。

实施代理的 `p3_instanceof` 定向测试为 5 通过、1 项 JDK 端到端默认忽略但显式运行通过；workspace `cargo check --locked`、rustfmt、OpenSpec strict 与 `git diff --check` 通过。丢弃、多用、旧值覆盖及 boolean→int 的拒绝来源已在该测试里固定；2.3 的完整预算/取消和相邻回归、3.1–3.2 尚未声称完成。

root 另从测试中的五组定长 `method_info` 补丁独立重建 pop、duplicate、stale、retained、boolean→int 变体；每份 class 的 SHA-256 与[拒绝边界记录](../../evidence/java-syntax-2026-09-22/instanceof/refusal-boundaries/README.md)相同，五个 Runner 均通过 `java -Xverify:all` 且输出逐字相同。因此可勾 1.2 的输入有效性和来源前提，不把“JVM 可执行”误当成 2.3 预算/取消已经完成。
