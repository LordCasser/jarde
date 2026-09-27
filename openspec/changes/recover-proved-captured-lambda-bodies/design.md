## Context

见 [proposal.md](proposal.md) 与 [DT-26 固定重放](../../evidence/java-syntax-2026-09-27/dt26-lambda-capture/report.md)。当前 `lambda.rs` 已按站点的 BootstrapMethods/MethodHandle、SAM 与实现描述符验证捕获数目和顺序；`build.rs` 的 lambda AST 已携带捕获表达式，却仍以调用 synthetic helper 作为 body。DT-25 的 [无捕获 helper 设计](../recover-lambda-synthetic-helpers/design.md)负责同类 helper 身份、方法体和整类引用清点；本项只有在该接缝被独立验收后才能实现。

## Goals / Non-Goals

**Goals:** 固定 `x -> x + base` 的一个 `int` 参数捕获，以及 `() -> this.number() + delta` 的 `this` 加一个 `int` 参数捕获；完整类源码可重编，捕获和 body 的求值时机不变，物理方法记录仍可查询。

**Non-Goals:** 任意捕获表达式、被重新赋值的参数、局部变量生命周期推断、泛型目标/适配、序列化 lambda、任意调用链/异常/分支 helper、把一个站点的成功当作整类省略依据。DT-25 的无捕获实现与 DT-27 方法引用各自独立。

## Decisions

1. **扩展同一 helper 证书，而非再造闭包后端。** 复用 DT-25 已验收的同轮 bootstrap 身份、物理方法解析、body 证明、全类引用 census 和原子提交。固定 classfile 的静态 helper 是 `REF_invokeStatic`，一个捕获值按实现方法第一参数映射，SAM 参数随后映射；实例 helper 是 `REF_invokeSpecial`，第一个捕获值是精确 `this` receiver，余下 `int` 捕获映射物理参数。仅在准确 handle kind、owner、descriptor、`private synthetic` 及相应 static/instance flags 闭合时准入。JADX 的 `CustomLambdaCall`/`makeInlinedLambdaMethod` 可参考参数重排和隐藏时机，但其 `DONT_GENERATE` 标志不能替代本库的全类证明。
2. **证明创建时捕获值。** 对固定首片，站点 SSA/栈操作数必须直接来自当前 receiver 与一个 `int` 方法参数，顺序与实现描述符完全一致。扫描完整 enclosing 方法的相关 slot 定义/写入，要求该参数满足 Java 源级 effectively final；不以仅在站点前未改写代替全方法证明。拒绝计算调用、字段读取、增量、副作用表达式、phi 或不明来源。此门槛允许在内联 lambda body 中使用 `this` 和参数名，因为 Java 编译器在创建函数对象时捕获这些稳定值，不会把一次性表达式推迟到每次调用。
3. **只替换 helper 内部计算的表达式。** 从 DT-25 已证明的完整 helper AST 中，把形参读取按上述映射替换为站点捕获值/SAM 形参，保留表达式节点的 helper 指令来源与站点来源。静态首片只准单一 `int` 直线算术返回；实例首片额外准许一个当前 receiver 上的普通实例 `number()` 调用加 `int` 参数。该调用原本在 helper 执行时发生，写入 lambda body 后仍在调用时发生，并保持虚调用 receiver；其余调用、异常处理或分支不在证书内。使用现有 emitter 的优先级/括号机制，不拼接或替换源码字符串。
4. **仍由类级全量清点决定省略。** 捕获版 candidate 与 DT-25 共用已选物理类中的所有普通 invoke、动态站点、方法句柄和不完整读取门槛。只在所有相关站点都完成捕获映射与 body 投影后一次性隐藏 helper 声明；任一额外使用或预算/取消停止都保留整组物理成员。单方法查询可以呈现当前方法的证据，但不拥有隐藏别的成员的权限。
5. **预算和对照边界。** 站点、SSA 定义/写入扫描、helper AST 替换、类级引用清点及输出均沿用现有预算维度并轮询取消。修后以固定 Java 8 原/JADX/Jarde 完整源码与同一 `Runner` 逐字比较 `7:-1`，同时核对无 helper 声明/调用和独立物理方法报告。负例覆盖效果性捕获、额外普通引用、错 flags/descriptor、参数重赋值及低预算。

只复用项目已有 reader、SSA/AST、来源和类级 adapter；没有需要引入外部库的能力缺口。另一条手写文本替换路径会丢失求值时机、来源与预算证据，因此不采用。

## Risks / Trade-offs

- [参数在站点后重赋值导致生成的 lambda 源码不能编译] → 以完整 enclosing 方法的 slot 写入证据限制首片，未知即拒绝。
- [`this.number()` 的 dispatch 或异常时机被移动] → 只在实例 helper 的当前 receiver 上复制该单次调用至 lambda body；不在创建点执行，也不折叠其返回值。
- [一个捕获源表达式重复求值] → 首片只接收直接稳定参数与 `this`，效果性来源单独拒绝。
- [DT-25 的接缝尚未通过 root 验收] → 此设计为后继任务，实施前核对 DT-25 实际证书与报告结构；若验收发现边界不足，先修正前置项或本设计，不新增平行类级机制。
