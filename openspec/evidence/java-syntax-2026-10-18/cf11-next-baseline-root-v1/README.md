# CF-11 下一份完整基线：TestNestedLoops3

root 已审查 Luna 的只读排程，完整上游 Java 和固定账本均有来源快照；14 个归档成员已逐字重开，见 capture-review.json。这是下一项基线的选择，不是新恢复能力验收，也没有提出实现机制。

首选原始 `jadx-core/src/test/java/jadx/tests/integration/loops/TestNestedLoops3.java`：两层循环、标签退出与内层 catch，原始 check() 有 c==302 断言。CF-11 仍为部分已测；当前仅确认没有找到这份原始完整类的合格对照，不能推断当前 Jarde 失败。TestNestedLoops5 仅为后续候选，不捆绑实现。

下一步冻结完整原始 class/profile，对同一输入完成原/JADX/Jarde 整类重编、运行、物理身份和 BCI map 对照；实际发现缺口后才取同次 IR 首拒并决定是否需要 OpenSpec。若已有能力足够，记录对照结果并取消实现提案。现有私有 source/driver 草稿不是编译或运行收据。

没有执行新 toolchain/runtime，没有修改 71/612 账本；JADX commit 只是固定账本引用，不冒称本次查询过 checkout HEAD。本备忘与 switch 产品交付独立。
