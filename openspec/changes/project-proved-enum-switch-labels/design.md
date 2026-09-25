## Context

见 `proposal.md` 与 `../../evidence/java-syntax-2026-09-24/enum-switch-labels/analysis.md`。`enumswitch@1` 只证明目标方法读取静态 `[I` 表，索引来自一个返回 `I` 的实例调用；它没有读取表所属类、枚举类和 `<clinit>`。完整类门面持有选定环境、物理身份与按需读取能力，现有接口初值和桥接工作也提供了“方法同次结论 → 类级证明 → 源码投影”的接缝。单方法恢复仍只陈述整数分派。

`EnumSwitchSubject.choose` 的 `getstatic` 在 BCI 0、`Hue.ordinal()` 在 BCI 4、`iaload` 在 BCI 7、`lookupswitch` 在 BCI 8；整数 1/2 到 RED/BLUE 的两条 `iastore` 在另一类 `EnumSwitchSubject$1.<clinit>` 的 BCI 19/34。将这两个常量对调后方法字节完全不变，合法 class 的输出从 `1|1,2|2` 变为 `2|2,1|1`，JADX 也对调标签。这是本项证明的必需反例，不靠编译器命名约定准入。

## Goals / Non-Goals

**Goals:** 对这一种有完整跨类证据的 Java 8 表分派投影枚举 selector 和常量标签；重编后保持选择、null 与 case 副作用；未知映射保留原整数分派与物理来源。

**Non-Goals:** 不恢复任意合成类源码、不作全 classpath 写入分析、不改变单方法 `enumswitch@1` 的整数语义、不把同名字段或 `ordinal()` 字符串当成证据；不宣称反射、包内外部代码对合成 `[I` 的突变也能被省略表的 Java 源码等价保留。

## Decisions

1. **跨类事实只在完整类装配时按需取得。** 从同次方法 IR/`enumswitch@1` 的结构化计划交接表 Fieldref、读/调用/selector BCI 与真实 SSA receiver；门面在选定 `LoadDomain` 中解析表定义和枚举类，确认 helper class 的真实 flags、表字段身份及枚举声明。索引调用必须由实际 owner/descriptor 与继承关系证明为该枚举实例的 `java/lang/Enum.ordinal():I`，不能仅因为调用名叫 `ordinal` 或返回 `I` 就把其他类认成枚举。只对命中规则的 member 做此读取；SingleClass、缺失/歧义、运行时开放或停止均拒绝投影。现有 resolver/reader/分析器承担解析与解析预算，不在 Java 输出层写第二套 classpath 查找。
2. **先证明 enum 字段、ordinal 与 `values()` 数组，再证明整个 helper 表。** `ACC_ENUM` 和字段名不证明 `getstatic Enum.A` 就是具有唯一 ordinal 的 A 对象。enum `<clinit>` 必须是完整单一路径，每个真实常量字段恰好由独立 `new Enum; dup; push(field-name); push(ordinal); invokespecial own (String,int) constructor; putstatic same-field` 初始化；`(String,int)` 构造器必须精确把两个参数转发给 `java/lang/Enum.<init>`。ordinal 必须按真实常量声明顺序从 0 连续到 N−1。`<clinit>` 尾部调用的必须是唯一、已解析的私有静态数组工厂；证明该工厂仅分配长度 N 的 enum 数组，并按 ordinal 位置逐一放入对应常量字段，返回同一数组供唯一的静态 final synthetic 数组字段保存。公开 `values()` 也必须是完整、无其他效果的该字段读取、`clone()`、同类型 cast、返回路径。这样 helper 所用的 `values().length` 才确定是 N，且枚举类本身的数组观察结果与投影源码一致；不能仅检查 `values()` 签名或把它的 IR 传入而不消费。任何 alias、重复/稀疏 ordinal、数组长度/元素/顺序改写、额外调用或 constructor normalization 都拒绝。这里没有证明 enum 类型全部行为；复杂常量参数、常量体和用户构造器形状暂不准入。
3. **证明整个表关系，再换任何标签。** 精确读取已解析 helper 的唯一 `<clinit>`：一个 `values().length` 分配的 int[] 唯一写入该字段，每个目标 case 整数来自唯一的 `enum.CONST.ordinal()` 下标，每条写入的 receiver 是该表，常量名对应已解析 enum 的真实 `ACC_ENUM` 字段。验证处理器只在对应写入周围捕获 `NoSuchFieldError`，无额外可观察效果、重复整数、重叠或未证明的写入/路径；helper 除 `<clinit>` 外无其他方法，目标类所有已读方法中的该表使用只允许来自已证明候选 switch。只需覆盖该 switch 实际使用的所有非 default 整数键；共享表上其他已证明写入可以存在，但缺失目标 key 仍拒绝。证明应由原 Code、CFG/SSA、异常边与字段事实交叉完成，不靠 `$SwitchMap` 名或常量顺序。
3. **同次 AST sidecar 做类级投影，不反解析文本或重跑方法。** `jarde-java` 只为拥有已证明 enum 表读的 class-source 方法保留有界的原 `Program` 与分派候选；门面按已选环境取得 key→常量后，调用 Java 层的窄投影/发射入口，替换 AST 中这一 selector/labels 并原子生成新的 `ClassSourceMethod.text`。原 `ClassSourceOutcome::Recovered.report` 仍是同次方法恢复的整数分派产物、来源与原规则记录；类源码投影另附物理表/目标类/BCI/映射的证明结果。不能从 `RecoveryReport.text` 查找/替换 `switch (...)`，也不二次分析或恢复方法。单方法 `recover_method` 不产生此 sidecar，沿旧整数路径。这需要一个窄的结构化类级投影接缝，而非通用跨类源码求解器或新 JVM IR 节点。
4. **先交完整证明，再计费发布。** 外部读取、候选遍历、sidecar 保留、key/enum 连接、新正文发射与来源逐项受现有工作/依赖/输出预算与取消约束。只有整项映射和整个方法文本可提交时才换标签；一个 case 缺证据则本方法全部保留原整数路径/拒绝。essential/all 的结构化证明相同，选择只影响详细记录，正文必须相同。完整类物理方法和原始 `RecoveryReport` 继续保留整数表读的原始事实，不把投影写回原字节码身份。
5. **不引外部反编译或泛型库。** JADX 是受控 oracle，不是运行依赖。现有 reader/resolver/Method IR 已能给出所需字节、选择和来源，外部库不能代替本项目选定环境与预算；新增一般图查询或持久缓存会扩大这个局部恢复项的维护面。

本地 JADX `2fb1b1638694` 的 `EnumVisitor` 与 `FixSwitchOverEnum` 提供了可迁移的处理顺序：先从 enum 初始化提取常量/数组字段，再从 helper 的数组写入收集整数→字段关系，最后在 `switch` 已建立 region 后同时替换 selector、指令 key 与 region 标签。Jarde 的同次 AST sidecar 沿用这个“先证跨类关系、后原子投影”的顺序。不能直接照搬其准入判定：`FixSwitchOverEnum.initClsEnumMap` 遍历 `APUT` 收集局部映射，`processRemappedEnumSwitch` 只检查每个 case key 是否有映射；`EnumVisitor.searchValuesField` 在多个数组字段中会优先选 `$VALUES` 名。这些步骤自身不证明表唯一写入、enum ordinal、数组工厂、`values()` 的完整效果或选定环境的物理身份。上述反例与预算契约仍按本方案逐项证明，不能因为 JADX 能发出 `case RED` 就视作等价。

## Risks / Trade-offs

- **表被合法字节码改写或与编译器模式仅外观相似** → 全表写入与路径证明，未知即保持整数分派；交换映射补丁要求标签随真实写入交换。
- **`NoSuchFieldError` handler 改变版本偏斜下的行为** → 证明 handler 的范围、类型、目标与正常写入各自对应；若异常边未知则不作 enum 投影。真实 `-Xverify:all` 对照普通与补丁类的原/JADX/Jarde 输出。
- **跨类依赖多读或输出半提交** → 复用同一环境/reader 的预算与停止；候选先证、方法文本后原子替换，取消与低预算测试同时检查正文和物理证据。
- **源码与原 class 的合成表在反射/外部突变下不可等价** → 明确本项只承诺已证受控调用路径；物理 helper 保留在分析报告/输入中，不宣称所有反射与包内其他代码的观察相同。
- **enum 常量字段可由合法 classfile alias，或 enum 构造器可改写 super ordinal** → 必须通过窄的 `<clinit>`/constructor 证明；`negative/aliased-enum` 在 `-Xverify:all` 下执行正常，却会让 direct-switch 重编输出不同，因此应稳定拒绝。此项对复杂 enum 形状是有意保守，不推断字段名或 `values()` 的声明顺序。
- **合法 enum class 可令 `values()` 返回 null、短数组或带副作用的数组** → helper 在分配映射表时会改变异常、长度或副作用；只证明常量字段与 constructor 不够。必须逐条证明 `<clinit>` 调用目标、数组工厂及公开 `values()` 的完整路径，并以 verifier-valid 补丁检验拒绝。
