## Context

两个已交付片的门互斥（proposal 已列代码行号实证）：`member_inner.rs:270` 要求父类构造器零参数（捕获证明），`facade.rs:4606` 要求 `field_count == 0`（父类实参转发证明）。混合形落在缝隙中，冻结 fixture `anonymous-super-args/AnonymousSuperArgs$1` 即该形（javap：`(String,int,String)` → `putfield val$captured(arg3)` + `invokespecial Base.<init>(arg1,arg2)`，无 `this$0`）。

**第一个取证义务**：读两个门的完整上下文，确认 (a) `prove_family_capture` 的"零参数"判据在何处消费——若它只用于确定"物理参数 == 捕获参数"的一一对应，则放宽为"有序子序列"后其下游（字段读取的词法替换、分配点唯一性）是否仍成立；(b) `project_class_source_anonymous_super` 的实参发射路径（4562 起，`root_ast`/`allocation_bci`/`constructor_bci`）当前假设"全部物理实参都是父类实参"，改为子集后其序保持与副作用计数如何调整；(c) 两片的原子发布接缝是否同一个（能否在同一次类源装配中同时应用两半证明），还是需两次独立发布——**这决定本片是一个投影还是两个投影的组合**。

## Goals / Non-Goals

**Goals:** 静态上下文、单分配点、super 实参与捕获值并存的匿名类内联为 `new Base(args) { … }`；捕获字段与构造器脚手架隐藏且可由 javac 重建；`anonymous-super-args` 完整源集可编译且行为一致。**Non-Goals:** `this$0` + 捕获 + super 实参三者并存（具名成员类形，属 inner-this 域的后续）；嵌套匿名类；多分配点；跨类引用；非 `structured` 正文；捕获字段的二次写入（既有片已拒）。

## Decisions

1. **合取并集，两门各放宽一处，不新建机制**：
   - `member_inner.rs:270` 的零参数门 → "父类构造器的实参是物理参数的一个**有序子序列**，且剩余物理参数逐一对应一个 synthetic 捕获字段的写入值"。
   - `facade.rs:4606` 的 `field_count != 0` 门 → "允许捕获字段存在，但每个物理参数的角色必须被**唯一证明**"。
   - 两个证明各自保持既有原子性与拒绝口径（任一子证明失败即整体回退物理文本），只是判据从互斥变为可并存。
2. **参数角色划分是本片唯一的新判据**（不是新机制，是对既有 SSA 消费点事实的分类）：每个物理参数按其 SSA 消费点归入且仅归入一类——(a) 流入 `invokespecial` 父类构造器的实参位 → super 角色；(b) 流入 synthetic 捕获字段 `putfield` 的值位 → capture 角色。拒绝条件：参数无消费、被两类同时消费、被同类的两个不同位置消费（如两次 `putfield` 同一字段）、或划分后 super 角色的实参序与物理参数序不一致（javac 保序，乱序说明这不是它生成的形态）。
3. **隐藏项与重建依据**：投影后隐藏的三项（捕获字段声明、构造器、字段写入）都由 `new Base(args) { … }` 让 javac 重建——字段由捕获的局部/表达式重新生成、构造器由匿名类语法生成、pre-super 写入由 javac 按其自身序发射（故 `ctor-reorder-dispatch-guard` 的过渡收敛在此形上被本片取代，而非冲突）。
4. **验收锚定**：`anonymous-super-args` 完整源集 `javac --release 8` 通过（当前退出 1）、`java -Xverify:all` 事件日志逐行一致（其 fixture 的 `event()` 日志记录构造顺序，是天然的行为对照）；两个已交付片的既有正例逐字不变；负例（参数无消费、双消费、乱序、二次写入、多分配点）保持物理文本。

## Risks / Trade-offs

- **两个门的放宽互相干扰**（`prove_family_capture` 的字段读取替换假设"物理参数 == 捕获参数"，放宽后索引可能错位）→ 取证义务 (a) 必须先确认下游；实现时用 fixture 的 `(String,int,String)` 三参形钉死"捕获参数是第 3 个而非第 1 个"。
- **实参序与副作用计数**（父类实参可能含方法调用，如 fixture 的 `text("super-label","explicit")`）→ 保持既有 super-args 片的"实参仍由原调用者 AST 发射、左到右求值与异常顺序不变"口径，本片只改"哪些实参属于父类"，不改发射方式。
- **投影组合的原子性**（若两半证明走两个不同发布接缝，可能出现半投影）→ 取证义务 (c)；若非同一接缝，则本片必须改为"先证明后一次性发布"，不得让两半各自发布。
- **与 ctor-reorder-guard 的顺序**：guard 片已合入后，混合形当前退回 verbatim（响亮失败）。本片是其终局解，实施顺序应在 guard 之后，且本片验收须包含"该 fixture 不再依赖重排"（即 ctor_order 的重排在该形上不再被触发或不再必要）。
