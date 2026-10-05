# 插桩定夺（2026-10-06，coder）——预审计复核与最小注入选案

本文件记录实现前的读码结论：root 预审计的三条断言逐条复核、两案（field.rs 侧接收闭包参数 vs 把
`array_of_value` 提为共享模块）的对照事实、以及最终落点。全部结论来自本工作树代码与 `javap -c -p`
实测；行号按当前工作树（预审计行号已漂移，机制为准）。

## 1. 预审计落点复核（机制为准）

| 预审计断言 | 复核结果 |
|---|---|
| 拒绝发出点 `build.rs:18997`（`fields.claim(at)` 为 None → 引注） | 成立，当前行号 **19169–19177**：`Some(Operation::Field { .. })` 分支的 `let Some((evidence, shape)) = fields.claim(at) else { self.fallback(…) }`，引注文本即巡查记录的 "the field access at BCI {at} is not one this run proved names the member its own receiver's type declares…"。实测 `RG.<clinit>` BCI 91 走该分支（`@bytecode 91` 引注） |
| 字段身份证明在 `field.rs:685 verify`，接收者类型只来自 `field.rs:786 stated_type`（只读 SSA 命名引用） | 成立：`verify` 的实例分支取 `operands.first()` 的接收者值，`stated_type` 只在 `ssa.value(v).ty()` 是 `Value::Ref(RefType::Named { .. })` 时答名；`aaload` 的帧类型是 `Produced::Ref` → `RefType::Unknown`（`jarde-jvm/src/frame.rs` 0x32 条目），故 `receiver_type = None` → 落入 `other` 分支的 "has the type this run does not state"（实测 `jarde-RG.txt` 记录的原话一致） |
| 既有通道 `build.rs:25393 array_of_value` 已对 `Operation::ArrayElementLoad` 递归数组操作数并降一维；参数数组（SSA 命名 `[LItem;`）与 newarray 局部链都覆盖 | 成立，当前 `build.rs:25570`：第一读是帧命名引用（`array_descriptor` → `(Type, dims)`），随后沿 `Duplicate`/`Load`/`Store` 移动链递归，`ArrayElementLoad` 分支 `dimensions - 1`；数组操作数是**调用结果**时同样由帧命名答（`EM.<clinit>` 的 `values()` 结果 `[LEM;`）。实测 RG（anewarray→局部链）、RK/RH/RL（参数）、EM（调用结果）四类来源全部可取组件类型 |
| `field.rs:540 plan` 签名已持有 `operations` | 成立（当前 `plan` 同时持有 `ssa`/`operations`/`budget`），但**单靠 `operations` 不足以**读数组形状：该读法还要走 `array_of_value` 的移动链与 `array_spelling`/帧命名拼写，属 `crate::build` 的所有物 |

复核同时确认两条决定实现方向的**结构事实**：

- `written_type`（`build.rs:25722`）的**第一读**就是"该值自己的数组形状"——`array_of_value` 成功即
  `array_spelling(&element, dimensions)`。实测 `RK.viaElemLocal` 呈现 `RK$Item local1 = arg0[0];`：aaload
  值经该读法得元素类型，声明层早已如此；本片只是把同一读法接到**字段身份证明**的接收者比较。
- 接收者的其余两处判据不能被新读法挤掉：`UninitializedThis`（实例初始化器在构造调用前的自类写）与
  owner-cast 父类写证明。实测复核：`UninitializedThis` 值的 `def` 是 `Definition::Entry`（
  `jarde-jvm/src/frame.rs` 的 `entry_frame`：`<init>` 的 local 0），而 `array_of_value` 只沿
  `Definition::Instruction` 展开、`Entry` 直接答 `None`，故新读法对该形恒无答案，原分支原样保留。

## 2. 两案对照（按最小 diff / 依赖方向 / 零回退面定夺）

| | (a) `plan`/`verify` 增一个接收者类型解析闭包参数，`report.rs` 用 `build::element_receiver_type` 提供 | (b) 把 `array_of_value`（或其所需逻辑）提为 jarde-java 内共享模块，`field.rs` 直接调用 |
|---|---|---|
| 依赖方向 | 保持 `field.rs` 不反向依赖 `build.rs` 的数组读法；调用方（`report.rs`，已是装配根）注入，`field.rs` 只声明"我需要一个接收者类型" | 需要移动 `array_of_value` + 其移动链 helpers（`single_stack_read`/`local_read`/`store_operand`/`instruction_at`）到中立模块，或新增转发模块；`build.rs` 内部 6 处调用点、`reuse.rs` 1 处调用点全部改路径 |
| diff | 3 文件 62 行（build.rs 新增 1 个 `pub(crate)` 入口 29 行含文档；field.rs 签名 + 判定 1 行 + 文档；report.rs 闭包 2 行） | 移动 80+ 行既有代码 + 7 处导入/调用点；`array_of_value` 的文档（"crate::reuse 也读同一通道，答案保持一次全身体读法"）随之搬到新家 |
| 证明面 | 与既证同形：闭包答案与帧命名答案经过**同一个** `match receiver_type` 比较（`stated == owner` / owner-cast / UninitializedThis / 否则拒） | 同 |
| 回退面 | 实测零（见 verification.md：RJ/RM/RO/RP 逐字节不变） | 同（同一读法），但改动面更大 |
| 与仓库既有惯例 | `field.rs` 已从 `crate::build` 取 `stack_operands`（小 helper），本次把"值→类型"的读法以参数注入以免读法本身跨模块 | — |

**定夺：(a)**。理由是按最小 diff 与依赖方向：`field.rs` 需要的是**一个事实**（该接收者的类型），不是
`build.rs` 的读法实现；`report.rs` 作为装配根已经同时持有 `ssa`/`operations`，注入点只有一处（生产
调用点全仓唯一）。同时按最小正确性补一条：数组读法只在**帧不命名**该值时被问（`stated_type(..)
.or_else(..)`），故命名引用路径逐字节不变。

## 3. 最终落点（机制）

1. `crates/jarde-java/src/build.rs`：新增 `pub(crate) fn element_receiver_type(ssa, operations, value)
   -> Option<String>`，紧邻 `element_of_dimension`/`array_of_value`——`array_of_value(value, 0)` 得该值自己
   的数组形状，`array_spelling` 拼成 Java 类型（与 `written_type` 第一读同一读法），仅 `Type::Reference`
   转回**内部形**（`crates::facts::internal_form` 的度量衡：成员 owner 是内部名）。不可证即 `None`；
   数组拼写（`Item[]`）永远不等于类名，无需额外判据。
2. `crates/jarde-java/src/field.rs`：`plan`/`verify` 增 `element_receiver_type: &dyn Fn(ValueId) ->
   Option<String>` 参数；`verify` 的接收者类型改为 `stated_type(ssa, receiver).or_else(||
   element_receiver_type(receiver))`。其余判定（`UninitializedThis`、owner-cast、拒形文本）一字未动。
3. `crates/jarde-java/src/report.rs`：`field::plan` 调用点构造闭包
   `|value| build::element_receiver_type(ssa, &operations, value)` 传入。

规则文本（`field.rs` 模块文档与 `verify` 注释）同步说明：接收者类型=帧命名；帧不命名的那个值由数组操作数
的自身形状命名；两者经同一比较，来源不可证则保持不声明。
