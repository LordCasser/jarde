# Outer.super 家族投影：推送前验收状态

Root 从当前源码重建 `jarde-cli`，对冻结 `OuterReceiverCases` JAR 执行 `class-source --policy plain-jar --class OuterReceiverCases --format json`，并分别导出 Jarde 的 `ReceiverBase`、`ReceiverMemberBase` 全类源码。所有 Jarde 文本未经编辑，用 `javac --release 8 -Xlint:-options -g:none` 完整重编，再以 `java -Xverify:all OuterReceiverCases` 运行，输出 `20:10:1:3`，与已冻结的原/JADX 对照相同。根家族投影状态为 `projected`，有 7 条派生来源；单测确认默认/all 模式的文本与派生表相等，并保留 root 桥和 child 调用的物理报告。

Root 也将冻结 `OuterSuperEffects` 三个 class 打包，直接导出 Jarde `OuterSuperEffects` 与 `EffectsBase` 全类源码。根家族投影状态为 `projected`，有 6 条派生来源；`Member.run` 中两次 `tick` 的次序与唯一性由定向测试确认。但未经编辑的完整 Jarde 源码在 `javac --release 8` 下失败：外层 `main` 的局部声明仍写 `OuterSuperEffects$Member member = outer.new Member();`，而家族源码声明的是嵌套 `Member`。这不是桥分派或参数顺序的失败，而是独立的成员类型局部声明拼写缺口；不得把此夹具算作三方整类重编通过，也不能因此勾选任务 3.1/4.1。后续按独立架构债务处理局部类型来源与源名映射，不能在本桥投影里做字符串替换。

本次 `cargo test --locked -p jarde --lib --quiet` 为 85/85；`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate project-proved-outer-super-bridges --strict` 通过。源级重载、泛型和 checked-exception 反例在[独立证据](../../evidence/java-syntax-2026-09-27/outer-super-source-binding/README.md)，Root 重跑了该证据脚本，原/JADX 四组 Java 8 重编及 `-Xverify:all` 运行一致，错误的 checked 重载变体按预期编译失败。以上仅为阶段验收，不代表 3.1/3.2/4.1 的完整签收。
