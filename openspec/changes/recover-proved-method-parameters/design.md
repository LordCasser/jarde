## Context

见 [proposal.md](proposal.md) 与[冻结对照](../../evidence/java-syntax-2026-09-27/em03-method-signatures/report.md)。`jarde-reader::attribute_facts` 已对成员属性建立有预算的类型化读取，但当前略过 `MethodParameters`；它不能塞进 `MethodCodeFacts`，后者的计费只覆盖 `Code` 的嵌套属性。class-source 在每个成员恢复前已经读取一次 `Exceptions`/`AnnotationDefault`。`src/class_source.rs::parameter_names` 和正文恢复目前只从 `Code` 的 LVT 构造名字；没有 LVT 就用 `arg<slot>`。`jarde-java::names` 已有单槽无范围名称的 `DebugLocal::named` 和统一 `NameTable`，无需另建名称重写 pass。固定 JADX 的 `DebugInfoApplyVisitor` 先比对参数个数，再给代码变量设置名字与 `final`；它没有替我们证明 class-source 发射的声明/正文原子性。

## Goals / Non-Goals

**Goals:** 在准确 Java 8 成员级参数属性的无 LVT 首片中，按 descriptor 的参数顺序和 JVM 槽宽把名字交给**同次**正文恢复与声明发射；`final` 只修饰准确对应的源码形参。属性只解析一次，保持各层预算、来源和停止边界。

**Non-Goals:** 通用 LVT/成员属性冲突策略、隐式/合成参数改排、畸形 Smali `Exceptions` 推断、为所有调用方重命名参数、重写既有异常声明逻辑。

## Decisions

1. **成员属性在 reader 解析，不让 `Code` 读越界。** 扩展既有 `AttributeFacts` 的有界成员属性读取，准确解析 `u1 parameters_count` 与每项 `u2 name_index/u2 access_flags`，保留原始顺序、可空名称及 flags；仅接受该属性所允许的 flags，重复/长度损坏由既有 reader 错误处理。由 class-source 按成员自身 shells 调用，避免第二次读和第二套计费。替代方案是在 `MethodCodeFacts` 扫它，但其注释和费用契约明确排除成员级属性；不采用。现有 parser 是本项目唯一有预算并保留物理身份的 reader，新增外部 classfile 库只会多建一份事实来源。

2. **先核对 descriptor，再映射至槽位。** class-source 只在本成员 `MethodParameters` 项数与 descriptor 形参数相等、每个待用名称安全、没有 `ACC_SYNTHETIC`/`ACC_MANDATED` 的隐式参数重排、槽位由既有 `parameter_positions` 唯一确定时建立此投影。`name_index == 0`、重复/碰撞的名称及不支持的 flags 不使正例部分成功。若本方法有 LVT，保持旧 LVT 路径；其与成员属性的冲突另审，不能由本切片静默选一份。其它方法不因一个成员的属性改变。

3. **在正文恢复前交接同一名称计划。** 将获证的无范围、按槽位名称交给既有 `RecoveryFacts`/`NameTable`，声明 `parameter_names` 从同一恢复事实取得名字；不能在恢复完成后替换源码文本，因为这样会漏掉正文、来源和别名位置。成员属性的 `ACC_FINAL` 则按同一参数顺序在现有 `arguments` 发射前附加 `final`。不要把成员属性伪称为 LVT 范围事实；仅使用 `DebugLocal::named` 的无范围路径，并把该来源区别记入解释/测试。若命名与正文来源无法统一，拒绝属性投影，不发布新 header。

4. **原子发布、窄回归。** 对参数属性解析、槽位映射、方法恢复和最终输出使用现有预算与取消；失败保留物理方法和拒绝原因。固定 `-g:none -parameters` 三方完整源码检查反射名与 `final`，再用无属性、多槽宽 `long/double`、错计数、空名、非法 flag、LVT 冲突和低预算/取消作拒绝或保守回退；`throws IOException` 和 class-source 既有回归保持原样。

## Risks / Trade-offs

- [只改 header 导致正文未定义名称] → 正文恢复前把获证名称送入同一 `NameTable`，全类 `javac --release 8` 作强制验收。
- [形参序号误当局部槽位] → 用 descriptor 的 `parameter_positions`，实例接收者与双槽 primitive 都作为拒绝边界/回归。
- [无效属性被误当源声明] → reader 完整解析，class-source 对项数、名称、flags 和冲突作原子准入；无法证明时保留物理来源。
- [额外属性读取改变预算或停止] → 只在声明了该属性的成员读取一次，同一请求支付字节费用；测试低预算和取消，不转成普通回退吞掉停止。
