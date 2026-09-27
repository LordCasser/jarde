# 独立恢复债务：原 probe 的共享辅助方法

原冻结 probe 的 `invoke(int,String)V` 在 `if (THROW_AT == site)` 内以三路条件创建不同运行时异常，随后共享一次 trace append 和 `throw LAST_FAILURE`。fresh Jarde 对该方法的 BCI 66 给出 `UncoveredBlocks [66]`；完整类输出可重编，但异常路径缺少原 class 的 `throw`。这不属于 `test(I)V` 的五行 finally 证书，也不证明其自身控制流可安全写成 Java。

后续应单列该多分支汇合与尾部抛出的 Region/Builder 恢复及来源证明，使用原冻结 probe 的 `invoke` 方法作为独立正例，并验证预算、取消和竞争分支。当前变更只用源级 helper 拆分的独立夹具验证 Test13 方法的三方完整类行为；原 probe 及其基线文件保持原样。
