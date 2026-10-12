## Context

见 proposal。父项证据包 `../recover-loop-body-try-catch/results/candidate-focused-and-comparison-root-v1` 的40命令均成功执行，但四份 v2 运行输出与原 class 不同：应逃逸的 Error 被省略，breaks 从0变1。385 lib 测试通过不证明该行为正确。当前 main 尚有循环候选未提交；本变更不覆盖或归档它。

实际 javac8 `exc()V`：new Error53/dup56，Builder57–76，getfield c70，Error.<init>(Object)79，athrow82。concat 因字段70报 `jre_concat_interleaved_effect`；init 随后正确拒绝“Builder实例不是构造实参”。init 已有 `verify_concat_arguments_metered`，核真实 String tail、唯一构造消费者、完整依赖与拥有区间，且会核整个 outer new 到消费者的精确异常表覆盖。field::Plan 在 concat 前产生，`claim` 已证明成员身份和 receiver；无需复制成员表。

## Goals / Non-Goals

**Goals:** 补齐直接实例字段 operand 的现有证明闭环，并通过原 v2 完整类门。求值位置而非纯净性决定许可；局部 receiver 包括 this 与其他精确类型的局部值，null 及 volatile 都须保持原操作。

**Non-Goals:** 任意字段表达式、receiver call/phi/cast、跨块链、getstatic 类初始化与 ConstantValue 被 javac 内联的证明、不相关旧扫描器的全面计费改造。原 init nested-instance-must-be-argument 门保持；无新 pass、公共 carrier 或依赖。

## Decisions

### 1. 复用字段事实，在既有 concat 入口认证新 operand

生产入口 `report` 已持有 fields、code、budget，将只读事实传入现有 concat 入口。新增核验只认 decoded instance FieldAccess::Read，必须与 `fields.claim` 的成员、receiver、读写方向一致；静态读不借此许可。字段 stack 输出是下一真实 append 的唯一实参且全方法唯一消费。instance receiver 来自紧邻该读取的 Ref Load：实际 Local read、同块、load 输出由该字段唯一消费，producer 未经过 dup/phi/cast/调用。不存在跨消费者重新读取 local 的写入间隔。其他 receiver 另有证明才扩展，不能猜测 this。

对 chain 的原 allocation/init/append/toString、overload、builder identity、aliases 和 owned 不变量全部沿用。新增 field 只在验证其完整依赖及归属后成为同一 chain 的 owned instruction，不以“这是一条读操作”放行所有 Field。字段 operand 的 receiver 及 result 与前后 append 顺序精确互证；保留其他不归属效果的 StatementFree 拒绝。

备选是仅准许非 volatile this 并传成员 flags。它不能解释现有有序 effectful invoke/field-copy 已可恢复的规则，且在原位一次读取下没有必要；因此不新增 volatility lookup 或 pure-field 注册表。实际读值与转换的顺序是必须通过的行为门。

### 2. 新许可先核完整消费区间与保护范围

只接受 chain String result 的唯一、同块、紧接 toString 的真实消费者；检查它确实使用该 value，保留当前可呈现消费形状。新字段许可涉及的 chain 全 span 和消费者逐条比较物理异常表 half-open 覆盖，比较有序 ordinal，不以同 SSA block 或 CFG catch edge 代替异常范围。预算停止原样传播。

内嵌 Error 则由现有 init 再核外层allocation/dup/constructor/sole consumer、真实 String实参依赖以及完整物理覆盖；concat 无权放宽 init 规则。build::concat_expr 仍按 append 序列渲染各 part，render_value 仍在实际消费点检查；chain/direct/derived origins 及 field read/receiver 锚点保留。

这一 child 不全面改写旧 ordinary concat 判定。实现优先在同一个现有入口为原字段拒绝补齐计费认证，所有新增扫描逐项 charge/poll，许可候选在全部成功后才发布。若需要 private error/上下文，仅复用当前 stop/refusal 表达；不能维护第二份部分成功 plan。旧入口的无预算扫描另作债务，不靠它预先做新增工作。

### 3. 在现有 AST 明确写出 Object 片段的即时转换

不能以左结合 `+` 文本直接证明 append(Object) 的转换时序。第二轮审计指出历史 javac indy 拼接曾延迟 Object 转换；若 toString 修改后续字段，这类延迟会改变读值。当前原始控制已实测本地Java8/23及Java23默认target六lane，plainPlus均与Builder相同，Java23的真实字节码在后续getfield前调用valueOf；不能声称本地编译器复现历史缺陷。这里明确转换用于锁定新许可链的原Object overload和求值位置。对本次新认证的 field-backed chain，在现有 Chain 上保存这份有序 operand 许可，build 为其中每个 Object overload 在原 part 位置使用现有 Invoke/Path/Cast AST 写 `java.lang.String.valueOf((Object) operand)`。转换的 String 是同一片段，不增加另一片段、不调用两次。明确 Object overload 防止 null/char[]/String 被选成不同 valueOf overload；wrapper 的来源派生自原 append，原 field/receiver primary origins 保留。

这样后续字段求值前必须完成该调用，不再依赖 StringConcatFactory 的转换调度；Object.toString 返回 null 的情况仍由 String 拼接按原 append(String) 的 null 行为转换。其它 primitive/String overload 按既有规则。仅新证据许可的链需要这项转换，不更改原已接受纯链、field-copy 或 conditional链文本；不为旧Object链问题作全局修复。若无法表达对应 overload 或保留转换位置，整个新增候选拒绝，不以原始 `+` 碰运气通过。

### 4. 验收包含转换对字段读值的影响

四组原 retry-v2 class 固定保留；CLI 重建后重跑父项40命令对照，比较原/JADX/Jarde完整类 raw。新增普通与 volatile int 字段、其他 local receiver/null、Object.toString 改变随后字段的顺序控制。编译生成的文本为 Java8/23都要保持事件与值；不能仅测试证明helper返回true。static/未证明member/额外使用/间隔effect/跨行覆盖/未知receiver拒绝，以及预算用尽与取消无部分owner，作为永久反例。

本地 JADX `ConstructorVisitor.java:65–105` 从真实 receiver NEW_INSTANCE 链组 ConstructorInsn，`InsnGen.java:727+` 从其参数值写 new。借鉴值图组合，不复制代码或引入 GPL 依赖；当前失败不是 lacking constructor AST。标准库、已有 Runtime 和 facts 足够，无新库维护/许可成本。

参考 [JLS15.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.7) 与 [15.18.1](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.18.1)：保留原片段顺序和转换；编译器优化不能被当作对照证据。classfile 解析、Java dialect、外部 javac/JVM验证、源恢复分别记录结论。

## Risks / Trade-offs

- Object 转换被延迟，使后续字段读到旧值 → 新许可链用现有AST显式Object valueOf；含 toString 写字段的执行控制，检查实际两个JDK的值与事件，不仅文本。
- 精确字段证据仅说明成员身份，不说明求值可移动 → 新许可独立核直接消费者、receiver值流、整个chain到消费位置的异常覆盖。
- 预算失败被当成字段不匹配 → helper 返回 Result，Stop先于候选commit；真实预算内部BCI测试。
- static常量被编译期内联改变类初始化 → 此许可不支持getstatic，独立记录后续证明债务。

## Migration Plan

Luna只在私有目录准备源码patch和回归；root对抗审阅并在当前main串行应用。构建遵守5GiB可用、target1GiB、1秒进程组守卫、default Rust stack、finally exact cargo clean。先 focused/source-map/stop 与四组 v2 整类对照，再 fresh workspace双seed、fmt/当前CIClippy/global strict、自身产品CI和干净main交付。若新证据推翻证明，修正设计后再实现，不缩小 v2 行为门。
