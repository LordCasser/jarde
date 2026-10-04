# Design — `recover-enum-string-field-name-generalization`

见 [proposal.md](proposal.md) 与 [取证证据](../../evidence/java-syntax-2026-10-04/enum-string-field-name-hardcode/README.md)。本文件把"已证字段名 → 发射处"的数据通路与不变量钉死，使实现者不必在关键位置猜测。

## Context（root 读码确认的两处证明与一处发射）

缺陷跨**两个不同函数**，二者当前都硬编码 `op`：

1. **构造器边证明**（`facade.rs`，`prove_enum_constructor_candidate` 一带，约 14094）：核对构造器 BCI 6 是 `putfield`，其 `EnumCodeReference::Field { owner, name, descriptor }` 的 `name == b"op"`。**这个 `name` 就是已证字段名**（构造器真实写入的目标），但被 `matches!` 消费后**丢弃**——承载结果的 `PendingEnumConstructorEdge`（约 13839）只有 `caller`/`call_bci`/`target_owner`/`target_descriptor` 四项，**不携带字段身份**。
2. **常量体组证明与发射**（`prove_enum_constant_body_group`，约 15495；发射文本约 15928）：另起三处 `== b"op"` 检查（约 15906 查源字段、15920 唯一性计数、15923 再核对），并在 15928 **字面发射** `"    private {}(java.lang.String arg0) {{\n        this.op = arg0;\n    }}\n"`。此函数的输入是 `relations: &[PendingEnumConstantBodyRelation]`（约 13812，含 `field_index`/`allocation_bci`/`constructor_bci`/`constructor_descriptor`/`subclass`/`group_shape` 等），**当前不含被写入字段的名字或 index**。

**故数据通路缺口是跨函数的**：字段身份的权威来源在 (1) 的 `putfield` 目标，而发射在 (2)，中间没有把它传过去。

## Decisions

1. **权威来源唯一：构造器字节码里 `putfield` 的目标字段。** 不用源字段表里的"某个 String 字段"猜测，不用常量名推导，不回退任何字面量。(1) 处 `matches!` 已绑定该 `name`（及可得的 field 常量池 index）——把它**保留**下来而非丢弃。
2. **优先传 `field_index`（u64，指向 `source_fields`/`field_headers` 的已证位置）而非裸名字字节**，理由：发射处 (2) 的三处检查本就在 `source_fields` 上按属性筛选（描述符 `Ljava/lang/String;`、owner==本枚举、非 static/synthetic/隐式枚举成员、`ACC_PRIVATE`、有 `declaration`、无 markers）——若传 index，可直接定位到那个已证字段并复用其 `declaration.name`，避免"按名字再查一遍"引入的第二套匹配逻辑（本会话已两次因"两套逻辑漂移"被迫加同形判据，见 `bridge_superclass_contract_generic` 的教训）。**若取证发现 index 通路不可得**（例如 (1) 与 (2) 的字段表非同一序号空间），退回传已证名字字节，但须在报告中说明为何 index 不可用。
3. **发射文本用已证名拼写**：15928 的 `this.op` 改为 `this.<已证字段名>`（从定位到的 `ClassSourceField.declaration.name` 取，与其余成员呈现同源）。**不得**继续硬写 `op`，也不得用常量名或类名推导。
4. **唯一性判据改为"被该构造器 `putfield` 写入的 String 字段恰一个"**：15920 的 `filter(name == b"op").count() != 1` 改为按已证 field 身份计数；其余属性检查逐字保留。15923 的 `op_field.item.name.raw().0 != b"op"` 改为核对"定位到的字段就是已证 `putfield` 目标"。
5. **错误文本去 `op` 化**：14094 的 `"the String constructor does not preserve Enum and op semantics"` 改为不含固定名（如 `"the String constructor does not preserve Enum and the assigned field semantics"`）。**该文本可能出现在既有测试断言里**——须核实并如实更新断言，不得为让旧断言变绿而削弱判据（与 `recover-bridge-superclass-header-precondition` 更新 BR$StrBox 断言同一纪律）。

## Goals / Non-Goals

**Goals**：带一个 String 源实参的枚举常量体投影，其能力边界回归 `recover-proved-string-arg-enum-constant-bodies` 的 Requirement 原文（"一个可无损拼写的 ASCII 字符串字面量源实参"），不含隐含的字段名 `op` 条件；非 `op` 名的同形枚举（如字段名 `t`/`label`/`value`）投影成功、渲染源集 `javac --release 8` exit 0、运行与原 class 一致。

**Non-Goals（proposal 已列，此处强调实现边界）**：不改 `prove_static_assignment_suffix` 的 `totalUnits`/`sumUnits`（那是**文档记载为窄首片**的独立证明，见 [hardcoded-identifier-audit](../../evidence/java-syntax-2026-10-04/hardcoded-identifier-audit/README.md)，非本缺陷类）；不放宽 String 可拼写性；不支持多个 String 实参；不泛化接口匿名路径的 `b"D"` 特化（属 `recover-anonymous-parameterized-root` 的登记后续）。

## 不变量（验收必守）

- **`TestEnums2a/DoubleOperations`（字段名恰为 `op`）呈现逐字节不变**——它同时是"名字恰为 `op`"与"名字由字节码证明"两种实现的共同正例，是本片最重要的零回退锚。实现者须用其 SHA 或逐字节 diff 证明。
- **`recover-proved-string-arg-enum-constant-bodies` 的 8 项任务与全部既有测试零回退。**
- **非 `op` 正例（1.3 新冻结）必须有 CI 测试引用**——这正是原缺陷能潜伏的原因（验收锚恰为满足硬编码而设计，无对照探针）。本片落地后该正例进 CI，则同类"名字硬编码"回归将来会被自动捕获。

## Risks / Trade-offs

- **风险：跨函数传 index 时两个字段表序号空间不一致**（(1) 的构造器证明与 (2) 的 body-group 证明可能用不同的 `field_headers`/`source_fields` 视图）。缓解：取证阶段先确认二者是否同一序号空间；若否，按决策 2 的退回路径传名字字节，并在发射处用名字在 `source_fields` 里唯一定位（此时唯一性判据仍成立）。
- **风险：唯一性从"名为 op 的字段恰一个"改为"被 putfield 写入的 String 字段恰一个"后，若某枚举有两个 String 字段但构造器只写一个**——新判据应接受（写入目标唯一即可），旧判据会因"名为 op 的字段"不唯一/不存在而拒绝。这是**放宽**，须用 1.4 负例（两个 String 字段且写入目标不唯一）确认边界：只有当**写入目标本身不唯一**时才拒绝。

## Open Questions

- 决策 2 的 index 通路是否可得，须在取证阶段确认（root 未逐行追 (1)→(2) 的调用链，只确认了两个函数各自的输入结构）。若不可得，实现者按退回路径做并在报告说明——**不因此停手**（退回路径是决策 2 已批准的）。
- `class_source.rs:8309` 的 `totalUnits` 引用是否与本 String-实参路径有耦合（root 初判无——它属 `prove_static_assignment_suffix` 的静态后缀域）；实现者取证时若发现耦合，停手报告。
