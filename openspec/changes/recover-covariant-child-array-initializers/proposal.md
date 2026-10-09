## Why

EM-18旧direct完整六类家族剩余三个grid方法仍拒绝，原因之一是子数组结构证明提前要求父组件类型与child实际类型相等，无法组合Java合法的数组协变。前片已恢复wrapper构造参数，下一步应复用已有ownership与赋值兼容证明，闭合这个确定的接缝。

## What Changes

- 子数组candidate保留真实ValueId、唯一parent store、区间与共同提交证明，不再用类型相等替代ownership条件。
- 呈现时沿用准确store BCI/source/target对应的既有平台数组关系与selected snapshot hierarchy，类型不明仍拒绝完整正文。
- 复用旧direct全部六类及双真实javac输入，完整生成源集隔离重编/验证运行；保留拒绝、预算、求值次序与真实来源。
- 先解除child-array失败，再用完整报告判断collectionGrid的Signature refusal是否仍是独立问题，不放宽泛型门槛。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：允许已有完整数组initializer证明组合类型不同但Java可赋值的子数组元素。

## Impact

主要涉及`crates/jarde-java/src/build.rs`现有child candidate证明，复用Builder与facade已有类型事实；相关focused及完整family回归。不新增crate、pass、AST、subtype walker、类型表或依赖。

## Prerequisites and Non-Goals

前置`recover-constructor-primitive-conversion-arguments`须以代码SHA `8f624ea64cd9d8e73810cc3a2384c773fa7f87bd`的CI全部成功验收后，才实施本片产品改动；CI37968418985已实际四job/48steps全部success，root已核对产品/冻结CLI身份。允许先对冻结CLI与输入做只读基线和规划，不能提前勾选前置验收。

排除一般alias/phi、跨block/非顺序store、Dex fill-array-data、BigDecimal平台扩展、member/statement-position news、类型系统重做及receiver-tail计费债务。不删除失败成员，不把局部method成功冒称完整family成功，也不宣称EM18整个单元追平。
