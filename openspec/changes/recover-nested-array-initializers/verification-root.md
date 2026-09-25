# Root 独立验收

root 在 agent 停止编辑后重建 CLI，最终构建 SHA-256 `bc09c736df5d923e30079bf844eed8f2f9c7320510e9d89793b7437e61cc4093`。源码只增加一条 Clippy 局部豁免注释后再次重建，嵌套、乱序、额外使用三类完整类输出均与首次独立验收逐字相同。[完整报告](../../evidence/java-syntax-2026-09-25/nested-array-initializer/jarde-after-nested-root-report.json)的 `dynamic` 和 `literal` 均 structured、零 fallback，分别覆盖 28/28 与 25/25 个由 `javap` 独立读取的真实指令起点，无额外 BCI。

root 从冻结源码分别重编原、JADX 1.5.6、Jarde 三份完整 `NestedArrayInitializer`，都通过 `javac --release 8 -g:none -Xlint:-options` 和 `java -Xverify:all`；三份重编 class 与冻结原 class 逐字节相同，SHA-256 均为 `c1ad514b609bc1eefeb0f5de73e7e622bd7fa07b34a31749a18e77fd9d74f24a`。运行输出两行完全相同：`[[1, 2], [3]]:3:123`、`[[a, b], [c]]`。Jarde 内外层使用已存在的 `NewArray` 类型/发射，三次 `element` 按 1→2→3 求值一次。

[乱序写入](../../evidence/java-syntax-2026-09-25/nested-array-initializer/ordering-jarde-after-nested-root.java)的原/Jarde 完整类也逐字节相同，SHA-256 `aeae53c70a3bab1e9e341829ad9cd2f55a18915b98d526a962654823aa7d4b15`，验证运行 trace `12`；JADX 同一 class 重编 SHA-256 `b7591a04901b623d936e340ef2cbec855dbff720b8ae64118e6575fbcddbd46a`，trace `21`。因此本次扩展没有引入 JADX `TreeMap` 排序的语义错误。verifier-valid [子数组额外使用](../../tests/fixtures/p3-nested-array-initializers/NestedArrayExtraUse.java)在原/Jarde 完整类中均输出 `[[4]]:4`，重编 class 逐字节相同；Jarde 保留读取子数组长度和普通父数组写入，不伪造嵌套字面量。

审读 `ArrayInitializers::prove` 的逆序候选与从普通消费者出发的闭合提交：子层单独候选不会独立认领父 store，父层必须准确绑定子层最终 SSA 值、终端 `aastore`、组件类型和物理区间；所有层仍走既有连续索引、唯一身份、handler、预算及深度门。没有新建公开 AST/IR。root 运行 `p3_nested_array_initializers`、一维初始化、部分维度分配和短路数组四个 test target（含 JDK ignored 项）共 15/15，通过；格式、`git diff --check` 与 OpenSpec strict 通过。普通 Clippy 通过；新扩展的证明函数只在该处加带理由的 `too_many_arguments` 豁免以保留显式证据参数，普通 Clippy 余 17 条均为相邻既有告警，严格 `-D warnings` 门禁仍未全仓闭合。

证据目录的 `nested-*`、`ordering-*`、`extra-*` 编译及运行文件、报告和 [CLI hash](../../evidence/java-syntax-2026-09-25/nested-array-initializer/jarde-after-nested-root-hashes.txt)可独立复核。agent 私有 Cargo target 已清理；root 私有 target 待本轮后续 OpenSpec 验收结束即清理。
