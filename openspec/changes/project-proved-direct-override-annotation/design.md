## Context

[EM-02 固定对照](../../evidence/java-syntax-2026-09-27/em02-modifiers/report.md) 已证 Jarde 六个完整目标类的运行语义与原/JADX 相同，唯一首片差异是同包 `PackageChild.onlyHere()` 缺 `@Override`。该注解的保留策略为 SOURCE，物理 class 文件不携带它。当前 `class_source.rs` 的单类方法声明只写本类 facts；`facade.rs` 已有同一请求内选择依赖定义、读取父/子类成员表并保留物理事实的路径。JADX `OverrideMethodVisitor` 先排除 private/static，再检查父方法可见性；其可见性规则可参考，但 Jarde 不能用同名猜测替代已选定义。

## Goals / Non-Goals

**Goals:** 在类源码最终投影前证明单一同 jar、同包直接普通父类的精确覆写关系；只给该子类方法加一次 `@Override`，并让原方法事实与推导依据可查。

**Non-Goals:** 接口实现、多级继承、跨包 protected/public 正例、泛型/协变返回、桥合并、Android/Smali 非法 flags 修复、对原始 SOURCE 注解的历史拼写推断。它们不因本切片标为已追平。

## Decisions

1. **投影证书位于类族组装边界。** 不修改 reader 或 `jarde-java` 的单方法 AST。`facade.rs` 在当前请求的选定定义及预算中读取准确直接父类，确认完整类头和方法表、两者同包、无泛型 Signature 及 bridge/synthetic 干扰；只在唯一同名同描述符的父/子方法均满足 Java 覆写条件时，向最终 writer 传递该子方法的父方法物理身份。单类 `MethodItem` 和物理注解列表保持不变。相比在 emitter 中凭方法名写注解，此处能区分定义解析和源级关系。

2. **只处理直接同包的准确描述符。** 父方法必须非 private/static/final 且为实例方法；子方法必须非 private/static/bridge/synthetic、非构造器，且访问级别不收窄。比较完整 JVM 描述符而不是只比参数名/源码字符串；重复、缺失或不完整都不投影。包名取两类准确内部名的 `/` 前缀。限制同包避免首片碰到跨包包私有及 protected 特殊规则；协变返回和泛型擦除另行证明。现有原始 `Runtime*Annotations` 中如已有合法 `java.lang.Override` 使用（非常规字节码）不得产生第二份，且不得把推导结果写进物理注解事实。

3. **复用最终方法文本组装。** 在 `ClassSourceMethod` 的最终文本前增加推导的注解行，保留原声明、body、markers 和来源映射；可用最小可序列化父方法身份字段记录该投影依据，而不增加泛化继承图或新的 pass。若父定义读取、预算或取消失败，沿用既有请求停止/拒绝，不发布部分注解或半份类文本。外部库不能替代本项目已有的 classfile 选择、预算和事实模型；JADX 源码仅作为算法条件参考，避免引入依赖与许可传播。

4. **固定三方验收。** [replay.py](../../evidence/java-syntax-2026-09-27/em02-modifiers/replay.py) 保留四个 JADX 测试 SHA-256，断言同包正例、private 与跨包负例；原/JADX/Jarde 六份完整目标类加共同 Runner 以 Java 8 重编并用 `-Xverify:all` 运行。另测缺父类、错 owner/descriptor、static/final、截断表与预算/取消。Smali `TestBadMethodAccessModifiers` 只记录为另一个非法输入边界，不计入这次正向验收。

## Risks / Trade-offs

- [父方法同名但并非合法覆写] → 要求准确直接定义、完整方法表、可见性和 flags，无法证明则不加注解。
- [推导的注解混入物理元数据] → 只写最终投影并记录父方法身份，不修改属性 facts。
- [请求中断产生半份类文本] → 与现有预算/取消、类文本原子发布边界一致。
