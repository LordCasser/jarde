## Context

见 `proposal.md` 与 DT-10 冻结证据。现有 `recover-proved-enum-constants` 已建立同次 prepared class 的类级证明：字段/方法表、`<clinit>`、构造器、`$values()`、`values()`、`valueOf()`、成员使用普查和私有 AST sidecar 在源码装配前合为一个证书。它只接受恰好两个 `ACC_ENUM` 字段和一个源级 `int` 构造参数；`ProvedOrdinaryEnumConstantGroup` 还把用户整数字段索引写为必有。普通空枚举的编译器构造器只有隐式 `String,int` 参数、没有该字段，值数组工厂仅建零长数组。匿名常量体另有独立证明，仍限定其原有范围。

## Goals / Non-Goals

**Goals:** 在原类级证明链上支持普通常量个数由物理表和预算决定、无源参数且无用户效果的隐式构造器，以及零常量的合法源码分隔；继续要求所有拟隐藏成员的完整使用普查和原子投影。通过冻结的 0、1、4 常量样本及已有两常量回归。

**Non-Goals:** 扩大匿名常量体、一般多参数枚举构造器、复杂初始化后缀、非 Java 8 编译器方言、重建源码原有空枚举是否手写分号。不同源码可编译为相同 class 时，只保证规范的合法拼写和行为。

## Decisions

1. **推广现有普通枚举证书，不新增并列恢复器。** `may_capture_group_code` 和 `prove_group` 对普通枚举不以常量数等于二放行；在同次字段/方法事实中按物理表收集常量向量、唯一合法名称及连续 ordinal。值数组工厂按已证常量数核对 `length`、逐项 `dup/index/getstatic/aastore` 和 `areturn`，允许零项。`<clinit>` 前缀按同一向量核对每次 `new/dup/name/ordinal/<init>/putstatic` 和一次隐式数组赋值；没有常量时前缀只含数组赋值。每条引用仍以物理 owner、descriptor、BCI 归属和完整 Code 为准。硬编码指令条数、BCI 和 `iconst_2` 不能推广为猜测；常量整数由已有 typed immediate helper 验证。备选的空枚举专用文本分支会绕过完整辅助方法/使用证明，拒绝采用。
2. **把源构造参数的有无表达为普通证书中的可选事实。** 保留现有带源 `int` 字段、两构造器委托和用户后缀证明；新增窄形状只接受唯一私有 `(String,int)V` 构造器，其完整 Code/SSA 除向 `Enum(String,int)` 原样传递注入参数并返回外没有效果、字段写入或异常处理。没有源参数时常量构造调用均使用该 descriptor、没有尾随实参；证书不捏造用户字段，类源码省去这条隐式构造器。现有必有 `constructor_field_index` 改为可选或等价的互斥模式，所有消费处显式核对，不以零索引或拒绝状态冒充成功。这里不是泛化任意构造器参数映射；真实源实参另归 DT-11。
3. **整类 writer 由证书决定分隔和消隐。** `ProvedOrdinaryEnumConstantGroup.constants` 按向量生成逗号列表；有用户声明时写 Java 必需的分号，零常量且无用户声明可省略分号。由证书隐藏已证的物理常量字段、隐式构造器、值数组字段/工厂及标准 API，`ClassSourceReport.fields/methods` 与各自 method-local source map 保持物理状态。用户方法仍按原顺序拼入，类源码只有全部检查和输出预算成功才替换；不能对旧 Java 文本做字段名搜索或删除。`recover-proved-enum-constant-bodies` 的恰好两项分支继续按原前提单独证明，不因普通枚举 arity 放宽而获准。
4. **沿用已有完整使用普查与拒绝。** 拟隐藏成员的任何额外直接或 handle 引用、损坏的 `$VALUES`/`values`/`valueOf`、异常/副作用或类表/Code/候选缺口都拒绝整组。只在选择完整类与 Java 8 profile 后恢复；reader 解析、runtime definition 选择、字节码验证和源码投影仍是分开的职责。JADX `EnumVisitor` 的线性初始化定位可作算法参照，但其按 synthetic/名称选择数组及直接 `DONT_GENERATE` 的捷径已被现有反例证明会改变行为，不能照搬。当前 reader、Code 候选和源码发射足以实现，不引入外部库或复制 JADX 代码；新库也无法替代物理使用关系与预算证书。

## Risks / Trade-offs

- `[零常量导致空向量被误当作无证据]` → 用字段表完整性、唯一 `$VALUES` 与全部隐式方法/Code 证明决定成功，空常量向量本身不是缺失；专测空枚举与空枚举带用户方法。
- `[把无源参数构造器误认成有副作用的用户构造器]` → 同时核对准确 descriptor、原始 opcode/调用目标、SSA/effect、异常表及方法恢复质量；任一多余指令拒绝。
- `[改变旧两常量或匿名常量体投影]` → 保留旧证书和专用分支，跑现有 `Stage`/`Measure`、委托构造器、用户 static 后缀、匿名常量体及额外 `$VALUES` 引用回归。
- `[常量数增大使循环或输出无界]` → 每项字段/指令核对与输出字节沿用同一预算，所有索引使用检查运算；停止不发布局部常量列表。
