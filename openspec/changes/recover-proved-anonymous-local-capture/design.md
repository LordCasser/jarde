## Context

DT-08 冻结对照见 [analysis.md](../../evidence/java-syntax-2026-09-27/dt08-local-capture/analysis.md)。当前匿名接口投影已具备唯一直接返回分配点、双方匿名 `InnerClasses`/`EnclosingMethod`、`Runnable` 抽象方法合同、整类引用普查、同次 child AST 和原子根文本替换。已有 `prove_family_capture` 则对 `this$0:LOuter;` 的 synthetic-final 字段证明构造器写入及全部 SSA 读取。`double` 参数捕获使用同类物理链，但 descriptor 是 `D`，根方法实参来自参数槽而非 `this`。

## Goals / Non-Goals

**Goals:** 在现有结构内让根参数与唯一 capture 字段、child 构造器参数和方法读取精确绑定，并以同次 AST 输出可编译的匿名接口源码。

**Non-Goals:** 不按 `val$` 名称、匿名类序号或字段顺序猜绑定；不处理多字段、局部变量经赋值后捕获、`Thread`/父类、非返回分配、泛型接口与桥方法。

## Decisions

1. **复用捕获证书的物理身份，不新增通用闭包机制。** 在 `member_inner` 已有 field/constructor/SSA 验证路径旁加入受限 primitive 参数形态：完整表、唯一 synthetic-final `D` 字段、准确 `(D)V` 构造器、唯一 `aload_0; dload_1; putfield; aload_0; Object.<init>; return` 及无 handler/额外写入；对每次读取证明 `this` 接收者、字段 owner/name/descriptor、结果 SSA 消费者。现有外层实例形态保持原判定。备选是靠 `val$d` 命名关联参数，缺乏来源保证，故拒绝。
2. **根方法与 child 证明在 facade 汇合。** 同次根 AST 的 `new` 唯一参数须是根方法的未修改参数槽，其 descriptor 与证书 `D` 一致；根方法完整 Code/SSA 与 owner 普查保证没有其它分配、引用、槽复用或创建时额外效果。物理 child 仍可查；根源码只在所有读取已准确替换且整个方法可呈现后提交。
3. **在 Jarde Java 层按 BCI 改写字段读取。** 延用当前 `ProvedCapturedOuterRead` 的精确字段身份和 AST read 匹配思路，把目标源表达式改为根参数名。若可用内部共用小函数，先抽出双方共用的字段读取定位；不要引入新的公开模型或重跑恢复。输出、分析预算和取消沿原路径收费。

## Risks / Trade-offs

- JVM 允许 synthetic 捕获字段在 `Object.<init>` 前初始化，但 Java 8 物理 child 源码不能照抄；只有根类匿名内联完全成功才可把本形态称为可重编源码，失败时报告不得虚称完整可编译。
- `double` 的 `-0.0`、NaN 等位模式对表达式改写敏感；固定两个值并加入带位模式断言的负/正例，参数本身必须是同一不可变值而非重新计算的表达式。
- 覆盖方法里不止一个读取或存在 method-handle/跨类引用时易漏改；完整物理使用普查与全部 AST 锚点消耗是提交前提。
