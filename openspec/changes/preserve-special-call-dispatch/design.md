## Context

动机及实测值见 proposal.md。`decode::invoke` 把 `MethodRef` 与 `InterfaceMethodRef` 合并成相同的 `CallTarget`；`Builder::call_expr` 对 Static 之外的类别都直接渲染 receiver。这丢失了非虚调用与接口限定的区别。原 change 4.3 的属主比较规则也不能区分父类和接口。

`MethodIr` 已持有本次读出的 `Arc<ClassFacts>`，其中有 `super_class` 与 `interfaces`；不需要重新读头、不需要 callee Body，也不用给 `MethodDeclaration` 再复制这些字段。当前类 `ClassMembers` 的 flags 已足够证明 private。

## Goals / Non-Goals

**Goals:** 在现有调用构造路径里选择正确接收者表示，以同源类头/池项/SSA 证明三种明确形式。

**Non-Goals:** 不启动 resolver，不做任意祖先搜索、别名分析或通用调用 IR 重构。非当前类 private、祖先跳层及其它未证明情形如实保留引用。构造器路径保持其现有规则。

## Decisions

1. 保留 `InvokeKind::Special`，将池项是否 `InterfaceMethodRef` 的原始区别交给 `CallTarget`。可增加一个命名清晰的布尔事实，不把 Special 拆成多个 dispatch 种类；不从属主名称猜接口。当前构造点很少，可直接改签名而无兼容适配层。
2. 为 `MethodIr` 提供借用现有 facts 的最窄访问器，供当前类 `super_class` 和 `interfaces` 使用；report/build 同源交接，不把类头复制到第二个缓存或从 class-source 外层反灌单方法结果。缺少事实保持缺少。
3. 入口 this 的证明使用 SSA：直接 receiver load 读取的是入口 Local(0)，方法确实是实例方法，且最终消费点的旧值检查仍成功。不仅比较局部文本是否叫 `this`。不跟踪任意 astore 别名或非平凡 phi；这些可以留缺口。复用现有 slot/value 查询，不建立新分析 pass。
4. 决策顺序：先维持 `<init>` 已认领路径；对其它 Special，当前类同名同 descriptor 的声明为 private 时保留实际 receiver；否则以池项类型匹配当前声明的直接父类/接口，并要求入口 this。证据不齐时返回构造错误，沿用现有 fallback/source-map，不能降成普通虚调用。
5. 在现有 ExprKind 中增加一项 `Super { qualifier: Option<String> }`，仅作为调用接收者。无 qualifier 写 `super`，有 qualifier 写 `T.super`。这样无需把关键字伪装成局部名字或普通类型路径；沿用 Call 与后缀发射规则，不新增语句、region、注册器。receiver load 的来源转交该节点。
6. 同类私有方法与其它对象 receiver 是必要正面对照。不能把所有“非 this 的 Special”都拒绝；旧规格此处需要修正。其它同类 non-private Special 也不能仅凭 owner 相同写 `this`，因为那可能重新引入虚分派。
7. 本地 AST/事实已足够，外部解析/反编译库不提供本缺口所需的额外事实，引入它们增加维护和许可负担。本项无生产依赖。javac/java 只运行自写 fixture；jadx 只作对照，已在本例出现错值。
8. 拒绝路径须闭合现有延期生产者协议。主代理实测把自写 fixture 的 privateHelper 改为 public、并把已有带副作用实参的 special 目标改到该同类成员：`java -Xverify:all` 接受且实参调用运行一次，首版修复却仅引用返回和外层调用，遗漏 BCI 1 的实参调用。复用 `quoted_bcis` 与已有 `deferred_producers`：语句调用拒绝不能只给外层 BCI，遇延期 invoke 时还须追溯其 receiver/args，沿用去重、深度界限和最终消费位置。不新增效果 pass；这是本项新增拒绝所需的最小闭环。

事实解析、JVM 验证、运行时解析和源码恢复分开：类头/池项是事实，选择规则只呈现有证据的 Java 形状；测试的执行相等不改变报告的 verification 平面，也不宣称所有 Special 都正确。语义依据：[JVMS invokespecial](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.invokespecial)。

## Risks / Trade-offs

- 类 super 与接口 super 混淆 → 同名实现分别返回 7/11 的执行反例。
- 异名调用碰巧返回同值掩盖虚分派 → 子类覆写与同名直接父调用测试，旧输出必须失败。
- 将别的对象写成 super → 同类 private(other) 与非 this super 的否定输入。
- 单方法与类装配读取来源不同 → 两个公开入口、class/jar 对照，同一 header 的事实且无需额外读预算。
- 与取负同时修改 build/ast/emit → 在取负实施者停止编辑后再派本项；文档和证据可并行整理。
