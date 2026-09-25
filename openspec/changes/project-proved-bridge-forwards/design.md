## Context

见 `proposal.md` 和 `../../evidence/java-syntax-2026-09-23/bridge-source-projection/analysis.md`。`jarde-java::bridge@1` 已从同一次方法 SSA/flags 识别可呈现的转发和被拒绝的附带效果，但 verdict 只用于方法体恢复。`ClassSourceReport::methods` 按物理表记录所有成员，`source_text` 目前又逐项写成 Java 方法，因此两个只相差返回 descriptor 的方法在完整类里冲突。类源码门面已经持有本类一次 prepared read、所有成员头、运行环境和各方法恢复结果；已有接口初始化工作也建立了只供类装配使用的同次候选接缝。

## Goals / Non-Goals

**Goals:** 让有已证继承需求的普通 Java 8 转发桥接在源文本中由其源级覆写承担，而物理 bridge 的方法身份、原报告及投影理由继续可见。准入需同时证明 JVM 原字节码效果与 javac 从输出声明重建的擦除入口。

**Non-Goals:** 不把所有 `ACC_BRIDGE` 当编译器生成；不修任意泛型 `Signature`、名称碰撞、跨未解析依赖的继承关系，亦不把有独立效果的桥接改名。方法的独立恢复仍如实呈现原体；本项只改变完整类组装。反射可观察但无法从 Java 源重建的桥接注解/属性不在正面准入内。旧 `present-proved-java-structure` 的“不隐藏 bridge”只界定当时 change，本次由新的三方 class 证据界定范围。

## Decisions

1. **复用同次 `bridge@1`，不再识别一遍。** 给 class-source 成员恢复传递结构化的 bridge 候选：原物理身份和 flags、调用 BCI/真实目标 owner-name-descriptor、是否由既有规则证明为单次纯转发、拒绝状态。当前 `bridge::plan` 对无 `checkcast` 的纯转发返回 `Ok(None)`，虽记为 `presented=true`，却不保存调用目标；正例恰是这一路径。因此必须在已有 shape 证明中让有/无返回值 cast 两路都保存同一结构化目标，不能只凭 `presented` 或可选的 `BridgeRecord.forwarded` 准入。`bridge@1` 的现有 plan 缺少的目标字段从它已读的 pool/SSA facts 填入，不解析 `RecoveryReport.text`、不依赖可选 `RuleDetails`，不二次运行 Builder。现有 `recover_for_class_source` 候选通道可扩展而不建立第二套 pass 或持久 IR。
2. **类级交叉证明是另一项必要条件。** 在已读的当前类方法表中，目标须唯一、非 bridge、非 synthetic、可在同一 Java 类中声明且完整恢复；其 name/参数 descriptor 与 bridge 相同，返回 descriptor 不同且可证明协变。目标上的 `final`、`synchronized`、`strictfp` 等由保留的源声明承担的属性可以保留；不能从源声明重建的合成目标要拒绝。桥接自身则要求已知可重建的 `public|bridge|synthetic` flags 组合，额外修饰符（例如 `final`）拒绝。转发调用须精确指向该目标，receiver/实参各只消费一次；桥接自己的 Code 无 handler、独立 effect、未解释的转换或会改变投影结果的属性。再在当前环境的已解析直接父类/接口中找到要求 bridge 擦除 descriptor 的实例方法，核对 Java 输出保留该继承声明、访问/覆盖规则与返回类型关系。对当前正例，`BridgeApi.get():Object` 与本类 `get():String` 的关系由直接接口头和 `Object` 根类型证明，不需要解析泛型实参。更复杂的继承/类型关系若现有 resolver 不能证明即拒绝，不凭桥接 flag 猜测。
3. **原报告与源码投影分层。** `ClassSourceReport::methods` 保持物理表序和每项原 `RecoveryReport`；已证明的 bridge 成员携带对目标方法的投影归属及自身真实来源。`source_text` 对它输出明确的投影注释而不再输出冲突的 Java 方法声明，源级目标方法仍只写一次。未证 bridge 保留当前保守呈现和明确拒绝，不通过改名或删体制造“可编译但少了 JVM 签名”的结果。`ClassSourceMethod::text` 与汇总正文的关系需一同更新，不能暗中让 JSON 物理项消失。
4. **按预算先证明再提交。** 当前类成员集合与被访问的继承头由既有环境/reader API 按需读取，受原请求的 loader/policy、预算和取消约束；不扫描全 classpath 或引入缓存。对候选、匹配、来源和新文本逐项计费并轮询。只有整项 bridge/target/继承组合与输出费用都确定后，才组装投影源码；任何中断不得留下省略 bridge 却未交代归属的部分正文。essential/all 选择只改变证据平面，不改变判定和正文。
5. **不用外部反编译/泛型库。** JADX 在正例能隐藏桥接，但副作用和孤立桥接负例显示“改名/合并”不能作为规则；外部库无法替代本项目已有的物理身份、环境和预算证明。新泛型解析器增加依赖与维护面，却不是这个直接接口擦除例的必要条件；以后需要解析更宽泛的 `Signature` 时另开 change。

## Risks / Trade-offs

- **标志真实但方法不是编译器可重建** → 同时要求纯转发、真实源级目标和已解析继承需求；`negative/` 与 `orphan/` 分别隔离效果和继承条件。
- **源码编译产生不同桥接目标或丢 erased 入口** → 对正例完整 Jarde class 重编，`javap` 核对重新生成的 bridge，直接调用、raw 接口和 `MethodHandles` 擦除调用均以 `-Xverify:all` 比原 class。
- **元数据被悄悄丢掉** → 对桥接专有且不能从保留的源级覆写重建的可观察注解/属性保守拒绝，物理方法及原报告保留；不把无法表示的元数据说成已恢复。javac 23.0.1 `--release 8` 的独立探针显示：给 `String get()` 加运行时方法注解时，生成的 `Object get()` bridge 也带同值注解。因此“桥接有注解”本身不是充分的拒绝理由；若本次不能证明注解在重编译后的复制关系，应以“复制关系未知”拒绝并单列扩展，不把它说成永远无法恢复。
- **跨方法/继承读取导致停止半提交** → 投影决定和输出先按预算完成，失败保持原成员证据与 stop，定向低限及取消测试检查全文而非局部字符串。
- **畸形继承环可能重读当前类** → 当前 `resolve_symbol` 从直接接口/父类开始，合法的正例能直接命中擦除方法而不重新读取 prepared class；但一个畸形父接口环可能在失败前经 `HeaderClosure` 再到当前类。读后检查 `reads` 可拒绝投影，却不能消除已发生的第二次物理读取。本项尚不能把这一边界宣称为“单次读取”已证明；若验收要覆盖畸形环，应在独立 reader/resolver 任务中评估以 prepared header 作为闭包种子的接口，而非在源码层再造继承遍历。
