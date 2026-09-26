# 4.1 家族源码投影验收

Root 用当前工作树重建 `jarde-cli`（SHA-256 `fd7ac577120fe309c55b39ce80d7509fde6ae88f37ae7bc67cb2b6d14eedb326`），对[阶段一冻结 JAR](../../evidence/java-syntax-2026-09-26/named-member-family-stage1/fixture.jar)执行 `class-source --class NamedMemberFamilyStage1 --policy plain-jar --format json`。根/成员物理方法仍分别为 3/2，双方 execution 为 complete，`member_family.projection` 为 `projected`。根文本只声明一次 `Member`，写出 `outer.new Member()`、`NamedMemberFamilyStage1.this` 和未改写的 `other.state`；成员物理文本仍含 `this$0`。Jarde 家族文本 SHA-256 为 `2892b9720cf60c238ba6207dfe84577dc9ef4e08efdd90ff36ccf10aa9f5f4a8`，essential/all 证据请求的正文相同。

原源码、冻结的 JADX 源码和当前 Jarde 家族文本各自用 `javac --release 8 -g:none` 重编，再以 `java -Xverify:all` 运行，均输出 `2011\n20\n`；冻结原 JAR 也相同。Root 独立用当前 CLI 重放，未借用实现代理的编译产物或输出。
Root 还给同一源码加上 `package p;` 重新编译成 `p/` 下的 JAR。Jarde 只写一次包声明，以 `p.NamedMemberFamilyStage1.this` 呈现捕获值；原/Jarde 两份 Java 8 重编类均通过验证并输出 `2011\n20\n`。隐藏捕获字段前另要求其 flags 恰为 `final synthetic` 且没有字段/类型注解属性，避免源级再生成丢掉额外物理修饰符或注解。

另在 [`member_family_identity.rs`](../../../tests/member_family_identity.rs) 构造 `FamilyEffects`：`outer.new Member(tick()).value()` 有普通构造实参的可观察副作用，空接收者有异常路径。原 class 由 `javac --release 8 -g` 编译；原源码、安装的 JADX 1.5.6 源码、Jarde 家族文本均用 Java 8 重编并通过 `-Xverify:all`，输出 `1:1\nNPE:1\n`。这证明空接收者抛错前没有第二次调用 `tick()`。JADX 源码 SHA-256 为 `241032f581e997a68bcc620e958c3b934410b8c8919700202a81591e3862a53a`；Jarde 文本 SHA-256 为 `0682e665be2eb05f7dfc48e58cc25667eeca3691d0d4e51990bfc9b8a637b7ce`。原 `run` 的物理恢复为 explanation-only；报告仍保留该标记和原方法来源，家族文本只在 `new@1` 重呈现为完整语句、无 fallback 后省去标记。

负例方面，冻结的 `Outer.super` 桥变体返回明确的 `synthetic Outer.super method bridge is outside the proved family projection`，根与成员物理文本均保留。关系错配、缺少空值检查、未恢复方法、方法体与输出预算停止在 12 项家族测试中维持拒绝，不发布局部嵌套源码。Root 重跑 `cargo test --locked -p jarde --test member_family_identity`（12/12）、`cargo test --locked -p jarde --lib class_source`（22/22）、`cargo test --locked -p jarde --test class_source`（47/47）、`cargo test --locked -p jarde-cli --test class_source_cli`（16/16）、`cargo fmt --all -- --check`、`git diff --check`、`openspec validate assemble-proved-member-class-family --strict`，均通过。严格 Clippy 仍被仓库已有的其他模块告警阻挡；本改动引入的一处 `len_zero` 已修正，屏蔽已存告警类别后的目标 Clippy 检查通过。

本记录只验收 4.1。家族文本的派生 source map（4.2）、已知家族外使用闭包（4.3）和最终整体验收（5.1）尚未完成；当前投影状态不声称这些事项已闭合。
