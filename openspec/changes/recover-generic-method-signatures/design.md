## Context

见 `proposal.md` 与 `../../evidence/java-syntax-2026-09-24/generic-method-signatures/analysis.md`。reader 已按成员身份读取 `Signature` 原字节；query 的 metadata xref 已实现 JVMS 4.7.9.1 签名语法扫描，但只返回所引用的类；完整类方法头目前仅由 descriptor 派生。架构不变量要求 Query 与 Decompiler 共享 reader/身份而不相互启动，故签名 grammar 的共用层须在 reader，不能让完整类源码直接调用 XRef。方法体恢复已经把条件值重建为 `?:`，基线中的 `choose` 正文可直接重编；但把返回类型改成 `T` 后仍需单独证明正文的源级类型关系，不能由当前擦除声明下可编译推得。

## Goals / Non-Goals

**Goals:** 在当前类源码请求内，让方法自己声明且源级可完整表达的泛型类型变量恢复到声明，实际 descriptor 与反射泛型元数据均通过重编验收。解析错误、擦除冲突、未知绑定或停止时保持物理描述及拒绝理由。

**Non-Goals:** 不由调用点推断泛型；不把类/字段泛型、局部变量泛型、复杂通配符或注解一并恢复；不修 `preserve-invocation-argument-types` 的 reference 转换拒绝。独立单方法报告继续以物理 descriptor 为准。复杂合法签名可以保守拒绝，不能输出一半。

## Decisions

1. **同一成员的属性与 descriptor 交叉证明。** 在现有 `read_member_attributes` 接缝按需取得该方法唯一 `Signature`，保留其原字节和物理成员身份；无属性时仍走 descriptor 路径。泛型声明只在语法完整、Java 8 可拼写、方法级类型变量全有定义且每个参数/返回擦除后逐项等于真实 descriptor 时准入。类型变量擦除使用第一类界、否则第一接口界、否则 `Object`；名字、界和所在位置都来自 `Signature`，不从字符串相似性猜。`Signature` 的 throws 列表可以为空，而同一成员的 `Exceptions` 仍列有普通受检异常（javac 23 `--release 8` 的 `<T:Number> T choose(T) throws IOException` 即如此）；空列表时保持已由 `Exceptions` 属性拼写的 throws，非空时逐项核对擦除及顺序，不把合法空列表误判为元数据冲突。对本次最小闭环只投影方法局部变量及非参数化 class 界/使用；解析器遇到其它合法结构应完整识别后作明确不支持结论，而非忽略尾部。这个边界避免类头尚未恢复泛型时写出游离的类类型变量。
2. **把已有签名 grammar 提炼到共享 reader。** `jarde-query` 已有 JVMS 签名扫描实现，但 Decompiler 不应依赖 XRef。将其签名语法与有界 cursor 迁至 `jarde-reader` 的按需纯解析入口；query 从同一解析结果遍历原有引用，类源码从该结果读取可投影的方法结构。不得在 `class_source` 里复制第二套未校验 grammar，也不把全域 XRef 扫描作为方法头的先决条件。`noak 0.7.0` 只给 `Signature` UTF8 索引，不解析方法泛型语法；新第三方库或 crate 不能减少当前接缝，故本项不加依赖。解析深度、签名字节、节点数由已有预算与显式递归界限制；query 的原类引用顺序须保持。

2.1/2.2 的 reader 接缝只回答「属性语法是否完整、变量作用域能否解析、每个位置擦除后是否等于物理 descriptor，以及显式 `^` throws 是否与 `Exceptions` 一致」。它不认识方法正文和 Java 源的可拼写性。合法复杂签名的结构保留在解析结果中；是否能完整写成当前类头，以及擦除虽匹配但 `Integer` 正文不能作为任意 `T` 返回，均由 2.3 的 class-source 投影决定。把复杂形状的拒绝硬塞进 reader，会让 query 丢失本来可查询的类引用，也是多造一层错误的全局语义。
3. **只改变完整类的声明投影，并证明正文仍合法。** `ClassSourceMethod` 保留原物理身份、`RecoveryReport`、body 文本和来源；已证签名作用在本次类级头部装配，参数名继续按 descriptor slot 派生，泛型类型只替换对应类型位置并加入 `<T extends ...>`。使用既有 `arguments`/`varargs_type` 和 type-use 注解接缝，若这些装饰无法在新类型上证明位置，拒绝整项投影。descriptor 擦除相同不保证源码正文可编译：例如字节码以 `Number` 返回一个 `Integer`，在 `<T extends Number> T` 下不能直接 `return`。因此复用已恢复 AST、参数/局部来源与 Methodref/SSA 事实，只准入能逐点证明的类型变量赋值、返回、表达式合流及正文调用绑定；不认识的表达式形状整项拒绝，不引入运行时 `javac` 或第二套泛型类型检查器作为产品机制。目标方法的同类调用也须证明重编绑定不会改变；本次可先对有未证明同类调用者的成员保守拒绝，不为泛型任务发明跨方法推断器。投影说明记录原 Signature、擦除核对、正文兼容性与拒绝状态；不能修改独立方法恢复报告。
4. **预算与原子发布。** 属性读取仍按现有 reader 计费；解析每个语法单位及投影文本、来源/报告按现有工作和输出维度计费并轮询取消。只有完整方法头与来源都准备且费用通过，才替换类文本中的该声明。停止不得发布半个泛型参数表或改变物理方法集合。essential/all 使用同一证明，正文相同。
5. **用重编语义约束源头设计。** 原、JADX、Jarde 三份完整类都以 Java 8 重编，普通调用与 `Method.getTypeParameters/getGenericParameterTypes/getGenericReturnType` 逐项对照；还要用有效 classfile 的改界/未绑定变量补丁测试保守拒绝。反射是这里的正向语义要求，不以“调用值相同”替代泛型元数据验收。

## Risks / Trade-offs

- **签名语法与 descriptor 冲突** → 逐参数/返回擦除核对；拒绝而不伪造泛型。有效但异常的 classfile 仍保留物理来源。
- **类变量、注解或复杂类型需要别处同时改写** → 本次限制到独立方法局部变量和完整可拼写的界/使用；其余明确拒绝并另列扩展，不把局部成功当整类等价。
- **泛型化会让原方法体不再通过源码类型检查，或改变正文/同类源码调用的重载与推断** → 对返回、赋值、表达式合流及调用逐点使用现有 AST/SSA 事实；任何未证明位置拒绝整项投影。正例只包含可证明的 `T` 参数合流及返回；以后可用现有 Methodref 证据逐调用放宽。
- **签名来自不可信 classfile** → 使用已有大小/预算界、语法全消费与递归深度限制，不使用无限递归或未检查的 UTF8 替换。
- **重复解析或反向依赖造成架构债** → 现有 query grammar 迁到 reader 成为唯一按需解析源，query 与类源码各消费结构化结果且不互相调用；xref 的原有类引用序列测试必须继续通过。
