# 2.2c 独立验收：同次初始化前缀

两条常量的待证关系现在共享一个私有 `<clinit>` 前缀结论。它从同次 Code 的原始指令和常量池读出 `new; dup; ldc(name); ordinal; invokespecial; putstatic` 的精确 owner、字段、实参、顺序和 BCI，并要求构造对象在对应字段写入处被唯一消费。随后核对同次 `$values()` Code 的两元素数组确实按序读取这两个字段、主类调用该工厂并写入 `$VALUES`。侧车记录字段写入、数组元素读取、工厂调用、`$VALUES` 写入和前缀结束的物理 BCI；2.1c 的字段名/ordinal 只作为被核对的预期值，不代替实际实参。

前缀和工厂 Code 均须完整、逐指令连续且没有异常处理行。前缀后的直线用户静态初始化可保留；分支、提前退出、未闭合尾部及直接重读常量字段或 `$VALUES` 会保守拒绝，因为这些形状的别名或重新进入路径尚未证明。冻结 `Op`、`Mixed`、`Plain` 的 Java 8 `-g`/`-g:none` 及带非空直线静态后缀的 `Op` 正例通过。等宽替换 name、ordinal、`$values()` 元素字段，多余 `dup`/局部保存或字段写入、缺工厂/不完整 Code 和尾部缺 return 的控制均拒绝。Stage/Measure 不进入本待证关系。

代理运行 `jarde --lib` 63/63、`class_source` 47/47、格式与 diff 检查。Root 审核指令白名单和来源后，补上扫描同次 Code 候选表的预算收费，独立复跑 `enum_constant` 32/32；本步尚未核验子类构造桥、正文和整组投影，主枚举仍为 `Refused`，没有发布 `Proved` 或类体。P1 golden 的已知基线失败仍按 [2.1c 验收](verification-2.1c.md)单列。
