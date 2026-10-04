## Context

匿名捕获投影的分派入口是 `src/facade.rs::prove_anonymous_capture`（17508，由 3407 与 4161 调用），按**捕获字段描述符**二选一：`b"D"` → `member_inner::prove_anonymous_double_capture`（541），其它 → `member_inner::prove_family_capture`（158）。两条分支都不接受混合形，且**全仓生产代码没有任何处理 `val$` 型捕获的证明器**（`grep -rn 'val\$' --include=*.rs crates src` 只得 `ctor_order.rs:247` 的名字模式回退）。

冻结 fixture `anonymous-super-args/AnonymousSuperArgs$1` 的字段是 `val$captured: Ljava/lang/String;`（非 `D`、非 `Lroot;`），故落入 `prove_family_capture` 并因描述符不匹配被拒。另一侧门是 `facade.rs:4606`（`child_facts.field_count != 0` 即拒绝），即 `inline-proved-anonymous-super-arguments` 排除"混入捕获值"的实现位置。

**故本片不是"两门各放宽一处"（先前描述不准确），而是：新增一条 `val$` 型捕获的证明路径（可沿用 `prove_anonymous_double_capture` 的结构骨架，把 descriptor 从硬编码 `b"D"` 泛化为按字段实际描述符）+ 放宽 super-args 侧的 `field_count` 门。** 这仍是"复用既有证明骨架"而非新建机制（SSA 消费点判据、字段读取词法替换、分配点唯一性都是既有件），但工作量大于纯接线。

**第一个取证义务（修正后）**：(a) `prove_anonymous_double_capture`（541–660）的哪些判据是 double 专属、哪些可直接泛化——尤其 `anonymous_double_constructor_shape`（759，6 指令固定形 `aload_0;dload_1;putfield;aload_0;invokespecial;return`）对 `(String,int,String)` 三参形**不适用**（指令数与 slot 宽度都不同），需按参数角色划分重写为按序核对而非固定索引；(b) `prove_family_capture` 的 `this$0` 判据（179–191）与新路径的边界——具名内部类形必须仍走原路径，本片不得误纳；(c) `project_class_source_anonymous_super` 的实参发射路径（4562 起）当前假设"全部物理实参都是父类实参"，改为子集后其序保持与副作用计数如何调整；(d) 两侧的原子发布接缝是否同一个（能否在同一次类源装配中同时应用两半证明），还是需两次独立发布——**这决定本片是一个投影还是两个投影的组合**。

### root 已核实的关键契约约束（2026-10-04，实现前必读）

**`MemberCaptureProof`（`src/class_source.rs:987`）当前无法表达混合形所需的参数角色划分**——它只携带单个捕获字段的事实：

```rust
pub struct MemberCaptureProof {
    pub field_index: u64,      // 单一字段
    pub field_name: String,
    pub constructor: PhysicalMethodId,
    pub write_bci: u32,        // 单一写入 BCI
    pub reads: Vec<MemberCaptureRead>,
}
```

即它只能陈述"一个捕获字段被谁读"，**没有**"哪些物理参数是父类实参、哪些是捕获值"的划分表示。而混合形的核心正是这个划分（fixture：`(String,int,String)` 中前两个转发 `Base.<init>(String,int)`、第三个存 `val$captured`）。

约束与选项（实现者须在取证阶段选定并在报告说明）：

1. 该结构是 `pub` + `#[serde(deny_unknown_fields)]` + `Serialize`，故**新增字段会影响 JSON 契约**。若新增，须用 `#[serde(default, skip_serializing_if = "Option::is_none")]` 保持向后兼容（既有消费者与 golden 不受影响），并核实是否有 golden 断言其序列化形状。
2. 波及面已量化：**约 12 处消费者**跨三文件（`src/class_source.rs` 3 处：971/8758/987；`src/facade.rs` 6 处：5023/17544/19731/22605/22807/24297；`src/member_inner.rs` 8 处含 9/163/312）。扩展契约须逐一核实不被破坏——这是本片的主要风险面，不是判据本身。
3. **备选**：不动 `MemberCaptureProof`，另用一个并列结构承载参数角色划分（只在混合形路径产生与消费）。若消费者波及面证明扩展代价过高，优先此路——它把新复杂度关在新路径内，不触碰已验收的具名内部类/double 两条路径的契约。

**故本片的真实工作量在"契约如何承载划分"，不在判据**（判据是对既有 SSA 消费点事实的分类）。取证义务 (d) 的结论会决定选 1 还是选 3。

## Goals / Non-Goals

**Goals:** 静态上下文、单分配点、super 实参与捕获值并存的匿名类内联为 `new Base(args) { … }`；捕获字段与构造器脚手架隐藏且可由 javac 重建；`anonymous-super-args` 完整源集可编译且行为一致。**Non-Goals:** `this$0` + 捕获 + super 实参三者并存（具名成员类形，属 inner-this 域的后续）；嵌套匿名类；多分配点；跨类引用；非 `structured` 正文；捕获字段的二次写入（既有片已拒）；多个 `val$` 捕获字段（本片验单字段形，多字段登记为后续）。

## Decisions

1. **新增 `val$` 型捕获证明路径（沿用既有骨架，不新建机制）**：在 `prove_anonymous_capture`（facade.rs:17563）的分派处加第三条分支——字段为 synthetic-final-instance 且名字为 `val$` 形（或按 `ACC_SYNTHETIC` 事实）时，走一个新的 `prove_anonymous_val_capture`，其判据沿用 double-capture 的结构（完整表、每方法有 Code、唯一构造器、无 MethodHandle 逃逸读、字段读取全部可证、SSA 消费点闭合），但 descriptor 按字段实际类型而非硬编码 `b"D"`，构造器形按**参数角色划分**核对而非固定 6 指令索引。`prove_family_capture`（`this$0` 形）与 `prove_anonymous_double_capture`（double 形）**保持逐字不变**，本片不修改它们的判据。
2. **放宽 super-args 侧的 `field_count` 门**：`facade.rs:4606` 的 `child_facts.field_count != 0` 拒绝改为"允许捕获字段存在，但每个物理参数的角色必须被**唯一证明**"（见决策 3）。
3. **参数角色划分是本片核心判据**（对既有 SSA 消费点事实的分类，非新机制）：每个物理参数按其 SSA 消费点归入且仅归入一类——(a) 流入 `invokespecial` 父类构造器的实参位 → super 角色；(b) 流入 synthetic 捕获字段 `putfield` 的值位 → capture 角色。拒绝条件：参数无消费、被两类同时消费、被同类的两个不同位置消费（如两次 `putfield` 同一字段）、或划分后 super 角色的实参序与物理参数序不一致（javac 保序，乱序说明这不是它生成的形态）。
4. **隐藏项与重建依据**：投影后隐藏的三项（捕获字段声明、构造器、字段写入）都由 `new Base(args) { … }` 让 javac 重建——字段由捕获的局部/表达式重新生成、构造器由匿名类语法生成、pre-super 写入由 javac 按其自身序发射（故 `ctor-reorder-dispatch-guard` 的过渡收敛在此形上被本片取代，而非冲突）。
5. **验收锚定**：`anonymous-super-args` 完整源集 `javac --release 8` 通过（当前退出 1）、`java -Xverify:all` 事件日志逐行一致（其 fixture 的 `event()` 日志记录构造顺序，是天然的行为对照）；三个既有捕获证明路径（`this$0` 具名形、double 形、无捕获 super-args 形）的正例逐字不变；负例（参数无消费、双消费、乱序、二次写入、多分配点、多 `val$` 字段）保持物理文本。

## Risks / Trade-offs

- **新路径与既有两条的边界**（决策 1）：`prove_anonymous_val_capture` 不得误纳具名内部类形（`this$0` 字段走 `prove_family_capture`）或 double 形（走既有路径）——分派判据须以字段描述符与 `ACC_SYNTHETIC` 事实为准，且用既有两条路径的正例 diff 钉死零回退。
- **构造器形判据从固定索引改为按角色核对**（取证义务 (a)）：double-capture 的 6 指令固定形（`aload_0;dload_1;putfield;…`）对 `(String,int,String)` 三参形不适用（指令数与 slot 宽度都不同）。新路径必须按"参数角色划分 + 按序核对"表达，不得复制固定索引——否则会对 slot 宽度不同的形（long/double 占两槽）误判。
- **实参序与副作用计数**（父类实参可能含方法调用，如 fixture 的 `text("super-label","explicit")`）→ 保持既有 super-args 片的"实参仍由原调用者 AST 发射、左到右求值与异常顺序不变"口径，本片只改"哪些实参属于父类"，不改发射方式。
- **投影组合的原子性**（若两半证明走两个不同发布接缝，可能出现半投影）→ 取证义务 (d)；若非同一接缝，则本片必须改为"先证明后一次性发布"，不得让两半各自发布。
- **与 ctor-reorder-guard 的顺序**：guard 片已合入（`fc868aba`），混合形当前退回 verbatim（响亮失败）。本片是其终局解，实施顺序在 guard 之后；验收须包含"该 fixture 不再依赖重排"（即 `ctor_order` 的重排在该形上不再被触发或不再必要）。
