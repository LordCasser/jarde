## Why

EM-18完整direct-new家族的五种wrapper构造参数含明确数值转换，现有decoder与Builder支持这些转换，但构造区间仍以StatementFree拒绝。前置数组组合片已使同输入原18腿8→16/18；继续补齐已有转换事实与构造表达式的接缝，可复用JADX classfile/Cast参考，避免新增机制。

## What Changes

- 在现有普通构造参数闭合证明中，仅接纳属于实际参数依赖的PrimitiveConversion，继续由已有Builder证明一元operand、源类别和目标cast。
- 保留转换链、真实constructor descriptor、求值次序与完整来源；不把结构事实当作正文呈现成功。基线已证JADX删除两种long经float/double往返的cast而改值，原程序为语义oracle。
- 普通构造入口透传本次共享Budget，直接调用已有verify_metered并原子传播Stop；必要地补齐该入口既有漏计，不新增feature计费开关或预扫。
- 对照双javac的完整wrapper家族及独立完整正例，复用15opcode数值回归，补共享reader、无关转换、Boolean及失败生产者控制。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整普通构造参数可包含已证明的显式数值转换表达式。

## Impact

`crates/jarde-java/src/init.rs`已有参数区间判据与meter，`report.rs`单一生产调用点透传Budget/Stop，既有decode、Cast、render_value及new_expr验证的复用，以及测试和完整源集对照。不引入crate、依赖、cast evaluator、pass或注册表。

## Prerequisites and Non-Goals

前置compose-constructed-reference-array-elements必须以最终本地门禁及代码SHA CI通过后再实现本片产品改动；当前仅规划。冻结其CLI2及真实双流，不回写历史失败。

排除checkcast、shift/bitwise、一般alias/phi、成员类专用构造白名单及statement-position-news、nested covariant child-array、BigDecimal平台关系、NaN payload保证与转换消除。普通new的共享参数必须先审查已有保存/拒绝路径，不能把数组caller的单次依赖证明泛化成普通new已经安全。
