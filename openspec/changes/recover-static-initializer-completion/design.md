## Context

失败证据在 `../../evidence/java-syntax-2026-09-22/casts/independent/with-clinit/`。扩展审计在 `static-initializers/`。`class_source::spell_method` 已将 `<clinit>` 写成static块；`declaration::plan` 已给出 StaticInitializer。`emit` 与 `emit_source_map` 都接收该声明和同一组Stmt，目前都依次写包装、语句、无来源的闭合大括号；无值Return始终打印 `return;`。

[JLS 8.7](https://docs.oracle.com/javase/specs/jls/se23/html/jls-8.html#jls-8.7) 禁止初始化块中的return，并要求块可正常结束。仅更改末尾书写即可覆盖已由当前区域恢复得到最外层末尾return的场景；它不是恢复任意提前退出的证明。

## Goals / Non-Goals

**Goals:** 用既有声明事实决定最外层尾部无值Return的合法拼写，来源落在真实文本上，commit/replay完全一致。

**Non-Goals:** 不删任意位置的return，不新增控制流变换，不识别更多try/loop，不恢复裸throw或合并字段初值。遇到其它缺口如final字段赋值限定符、synthetic声明冲突，记录后单独处理。

## Decisions

1. 在已有emitter内共用一个最外层body入口，代替commit/replay两处相同的包装/语句/闭合写入链。只有已知StaticInitializer且顶层最后一个Stmt是Return{value:None}时，前缀照常发射，尾部完成映射到既有闭合 `}`。其余输入走原流程，不猜测没有声明事实的成员是否clinit。不能在class-source或CLI上删字符串行。
2. 沿用末尾Return的OriginSet，使用既有node/put写出闭合，保留成员身份、原始及derived来源。空文本不产生来源段，因此不靠“不写return但记录空span”蒙混。输出预算照实际字节收费，来源replay照既有分段收费；两遍共用决定才能通过逐字一致性门禁。
3. 不改变AST、Program或输入类型。Return继续表示字节码终止；Java源码的初始化块完成是这个上下文中的文本投影。body中其它Return（包括分支/try内和非尾部）不被本规则消掉：它们有独立控制流意义。本项不承诺那些输入已可编译，测试须保留其终止/拒绝来源与后续效果，不能以删除所有return假通过。
4. 不把被省去的Return再计成“实际发射的Java语句”。已有字段写入/调用/控制流仍按原计数；仅含Return的空初始化块投影后是Produced/Structured、ExplanationOnly，闭合仍有来源。这遵循主spec的content定义，不添加新状态、空分号或虚假计数；class-source的“no statement”展示措辞不在本项重设计。
5. 回归复用p3_patterns既有static_initializer_class真实class生成器与emitter单元边界；六类source-only输入在/tmp编译做整类JDK对照，无需再增加永久class或新语料分类。
6. 无外部依赖。声明读取、AST、节点来源及formatter都已存在，没有缺失的库能力；引入语法库既无解析工作可替代，还会增添维护、许可与预算成本。本项只改变source recovery；parse、dialect、runtime和verification/compile平面的契约不改。JDK执行只针对授权的自写样例，不作为引擎自动执行能力。

## Risks / Trade-offs

- 隐去分支内提前return会错执行后续写入 → 只处理最外层最后一条；用合法JVM提前返回变体和普通void/构造器对照钉边界。
- 文本与来源两遍漂移 → 共用body formatter，并测试终止BCI映射到实际闭合字符、成员身份和来源预算停止。
- 空块统计看似回退 → 按实际文本保持旧content契约，不把Produced/Structured推断成ContainsStatements。
- 更短输出使预算断点移动 → 验证停止仍无部分产物，记录必要的实际断点变化，不调大预算绕过错误。
- 条件/异常/循环中的其它恢复缺口混进来 → 审计逐例区分；只承诺现有区域已呈现且末尾符合条件的输入，拒绝区域留原来源。
