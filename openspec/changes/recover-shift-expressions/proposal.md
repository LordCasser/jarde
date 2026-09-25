## Why

六个 JVM 移位指令尚未进入 Java 恢复事实，常见 `<<`、`>>`、`>>>` 因而被引用。root 已独立编译不含位运算与数值转换的 ShiftCore：723 项原 class/JADX 结果一致，jarde 产生 29 处引用且完整 javac 失败；证据在 `../../evidence/java-syntax-2026-09-22/shifts/root-core/`。

## What Changes

- 忠实恢复 int/long 的三种移位，保持左侧独立提升得到的结果类型、距离掩码语义和操作数顺序。
- 复用现有二元表达式、分组、来源、预算与失败生产者路径；不把移位交给普通二元数值提升。
- 固定完整类的负值、边界距离、byte/char 提升、嵌套算术、局部与效果对照，以及 boolean 形状等不能直接拼写的明确边界。
- 生产实施排在已证实错值的求值顺序与方法引用适配之后，使用届时已验收的共享值保存；数值转换、位运算、复合赋值美化及一般循环/guard 扩张不在范围内。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在已接受区域中恢复六个移位指令，保持整数宽度、求值顺序、分组与已有拒绝及产物契约。

## Impact

涉及 jarde-java 的 facts/decode、BinaryOp/类型、build 消费和统一 emitter，必要时仅在既有纯值区域名单增加该事实。沿用 reader/JVM 的原始指令与 SSA，不改外层接口、不引入 crate、类型求解器或额外 pass。验收通过受控自写 Java 输入进行，产品不执行目标代码。原 ShiftAudit 的位运算组合及 ShiftDistanceBoundary 的 `l2i` 继续单独记录，不能承诺本项自动覆盖。
