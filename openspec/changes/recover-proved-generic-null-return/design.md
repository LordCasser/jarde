## Context

参见 [提案](proposal.md)、[规范](specs/java8-recovery/spec.md)和[冻结三方证据](../../evidence/java-syntax-2026-09-27/dt16-generic-null-return/report.md)。reader 已将 `<T:Ljava/lang/Number;>()TT;` 对 `()Ljava/lang/Number;` 做作用域和擦除证明。`jarde-java::generic_return_candidate` 从同轮 Program/SSA 产生 `GenericReturnCandidate`；`class_source::project_method_signature` 已把候选与方法 Signature 共同门控。当前 null 直接返回没有候选，且 `generic_method_declaration` 只准入静态参数返回。

## Goals / Non-Goals

**Goals:** 对顶级、非泛型、直接继承 `Object` 的普通类中的无参数实例方法，只有方法自身唯一 `T extends Number`、真实正文为无效果 `return null;`、无未知绑定及全部事实完整时，一次发布精确泛型方法头。保留方法体、物理身份和独立报告。

**Non-Goals:** 推广任意返回表达式、多个/类级类型变量、方法参数、泛型 throws、参数化界、接口覆写或继承、同类调用目标重绑定、DT-17/18 的集合类型，以及新的通用 AST pass。

## Decisions

1. **扩展既有同轮候选，而不从输出文本倒推。** 在 `GenericReturnValue` 的现有接缝表达无参数的 `NullLiteral`；只有 Program 单语句、非 ragged，Code 完整且准确 `aconst_null; areturn`，无异常处理器、SSA 单块无 phi、对应操作无局部读写/抛出时才产出。Code 的来源 BCI、SSA 效果和 AST 返回必须指向同一次真实读取。任何额外指令、转换、调用或隐藏副作用拒绝候选。这是一项局部形状，避免新 parser 或独立 class-source 扫描。
2. **签名与 Java 合法性沿现有门证明。** reader 唯一 `Signature` 的类型变量作用域和擦除必须精确；首片要求 `()Number`、方法自有单一 `T`、第一界为 `Number`、返回 `T`，方法无参数、实例、公有，类为顶级普通 `Object` 直接子类且无接口/类签名。继续检查注解、flags、同名重载与本类 `Methodref`；缺一项沿现有 `refuse_generic` 保留擦除声明。`null` 可赋给引用型 `T`，无需假设运行时值来自参数；不把 primitive 返回放进此分支。
3. **原子发布及验证。** 类级签名投影仍经 `project_generic` 整项提交和计费，停止时没有半个方法头；独立方法恢复结果仍以物理 descriptor 为准。用同一个外部 consumer 完整编译原始/JADX/Jarde 类型，运行 JVM verifier 和反射泛型元数据；负例用真实 classfile 变体区分“可解析但正文或绑定不能证明”和“预算/取消停止”。

## Risks / Trade-offs

- `return null;` 最终文本可能掩盖了检查、调用或异常路径 → 同时证明 Code、AST、SSA 和效果，拒绝所有非精确两指令正文。
- 只按签名输出泛型头可能改变同类调用/重载绑定 → 首片排除同名重载、本类 `Methodref` 及继承/覆写关系；外部 consumer 直接重编验证。
- 范围窄于 JADX 能输出的其它泛型方法 → DT-16 的 `TestGenericsInArgs` 及成员界形态另测，不把本片称为整个单元追平。
