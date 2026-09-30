# TWR saved-return 声明按保存值类型拼写切片（`recover-saved-return-value-typing`）证据 — 2026-10

[CF-17 巡查 README](../README.md) 登记的"新登记巡查点"实现侧证据：TWR saved-return 局部声明按保存值的生产者类型拼写，`T2.voidBodyReturnInside` 由不可编译的 `Object local1 = "in"; return local1;`（方法返回 `String`）改为 `java.lang.String local1 = "in"; return local1;`，整类 `javac --release 8` 一次通过。

## 诊断（tasks 1.1）

复现命令与基线：主线 `0a6bb340` 构建 `target/debug/jarde-cli`，`class-source --input …/fixture/T2.class --policy single-class --class T2`（基线全文 [results/T2-class-source.base.txt](results/T2-class-source.base.txt)）。T2.class SHA 见 `results/` 下 SHA 记录，与巡查冻结一致（`d702ff36…`）。

**决策点定位**：T2 saved-return 槽（slot 1）不落 `NullableResourceFinally` 的两个专用分支（声明写入 BCI 14 在保护体内，`write.at < body.0` 不成立；slot 3 分支限另一布局），最终走 `decide_types` 的公共回退（`build.rs` `Ok(Some(ty)) => Decided::Type(ty)` 分支的 `written_type(ssa, operations, write.written)` 调用）——**回退本身已调用 `written_type`**，"未调用 written_type"与"硬编码"两个候选根因排除。

**Object 来源**：三候选中第三个坐实——**SSA 值本身宽型**。`frame.rs` 的 `apply_constant`（`PoolEffect::Ldc` 分支）对 `ldc` 的 `String`/`Class`/`MethodType`/`MethodHandle` 常量一律推 `Value::Ref(RefType::Unknown)`（"The instruction names no class, so the reference stays unknown"），`value_type` 把 `Ref(_)` 拼为 `Object`。与 Test2 已修的 `array_of_value` 同族：值类型需从**值生产者**细化。

**第二层事实（临时插桩证实）**：写入 slot 1 的 SSA 值的 `Definition::Instruction { bci: 14 }` 是 **store 自身 BCI**，`operations.get(14) = Store{slot:1}` 而非 `ldc`——SSA 把 store 读到的栈值定义在 store 处，须经 `store_operand` + `instruction_at` 追一步到 `Push(String)@12`（`array_of_value` 的 `Operation::Store` 臂注释明说同一形态："A value a slot holds is the value the instruction that wrote the slot takes"）。

**实证矩阵（变体冻结 [Wtyping.java](Wtyping.java) → [original/Wtyping.class](original/Wtyping.class)，javac 23.0.1 `--release 8 -g:none`；基线 [results/Wtyping-class-source.base.txt](results/Wtyping-class-source.base.txt)）**：

| 变体 | 保存值生产者 | 实现前 | 实现后 |
| --- | --- | --- | --- |
| `stringLiteral` | `ldc "in"`@12 | `Object local1 = "in";` ❌ | `java.lang.String local1 = "in";` ✓ |
| `intLiteral` | `bipush 42` | `int local1 = 42;` ✓（从未坏） | 逐字不变 |
| `constructorValue` | `new StringBuilder` | `java.lang.StringBuilder local1 = …` ✓（从未坏） | 逐字不变 |
| `callReturn` | `invokestatic valueOf` | `java.lang.String local1 = …` ✓（从未坏） | 逐字不变 |
| `nullValue`（对照） | `aconst_null` | `Object local1 = null;` ✓ | **逐字不变（规范钉死）** |
| `classLiteral` | `ldc (Class String)`@12 | `Object local1 = java.lang.String.class;` ❌ | `java.lang.Class local1 = java.lang.String.class;` ✓ |

坏的**只有 ldc 引用常量**；int/构造/调用保存值的 SSA 类型本就精确（`Value::Int`/Named），从未坏。类字面量分支曾按 `ConstantValue::Class{ty}` 所指类型误拼，变体矩阵当场暴露（`java.lang.String local1 = String.class`），修正为常量自身运行时类型 `java.lang.Class`。

## 实现（tasks 2.1）

`crates/jarde-java/src/build.rs`：`written_type` 在 `array_of_value` 之后新增 `constant_of_value` 生产者分支——仅当值自身 frame 类型为 `RefType::Unknown` 且定义为 `Instruction` 时，经 `Push(String)`→`java.lang.String`、`Push(Class)`→`java.lang.Class` 细化，`Store` 臂按 `array_of_value` 同款 `store_operand` 步进（`MAX_VALUE_DEPTH` 防环）；`null` push、phi/caught/跨块值一概不细化，落回既有 `value_type` 拼写。**不加新机制、不动 finally 各证书的专用分支**；细化失败回退即既有回退通道本身。

- 拼写约定：声明类型按本 crate 既有声明通道全限定（`java.lang.String`，与 `callReturn` 变体实现前的既有输出一致）；类源输出本身声明"no imports"，简单名 `String` 不在本 crate 拼写约定内。
- 既有 null 期望 `crates/jarde-java/tests/p3_java_recovery.rs` 两处 `Object local1 = null;` 逐字不变（STRAIGHT_LINE 的 `aconst_null`，null 无窄型）。

## 前/后输出（tasks 1.2/2.1）

`results/T2-class-source.{base,after}.txt`、`results/Wtyping-class-source.{base,after}.txt`（`class-source` 全文捕获）。排除计时噪声后 diff 仅目标行变化：

- T2：`voidBodyReturnInside` 一处 `Object→java.lang.String` + 派生字节计数/segment/digest；
- Wtyping：`stringLiteral`/`classLiteral` 两处；`intLiteral`/`constructorValue`/`callReturn`/`nullValue` 逐字不变。

编译（`results/javac-check/`）：T2 恢复文本 `javac --release 8` 一次通过并运行（`done/in/done/done`）；Wtyping 六成员中唯一不可编点是 null 对照（`Object local1 = null; return local1;` 对 String 返回位）——规范钉死的既有拼写（spec scenario "null 与既有输出不变"），将该成员返回位机械调整后整类编译并运行六行输出与原类一致，证明五个非 null 成员类型全部可编。

## 三方对照（tasks 3.2，`results/three-way/`）

fixture 冻结两版如实记录：首轮冻结（SHA `cef98ed8…`）承载 base 捕获；为注入异常腿补 boom 钩子（`close`/`touch` 在 `Wtyping.boom` 时抛 ISE，V17a 同构）重冻结为 `59dbeded…`，六成员保存值方法 BCI 布局不变（javap 复核 `ldc@12; astore_1@14; close@15..16; areturn@20; handler@21` 同构），after 捕获已在第二冻结上重取。驱动源 [WtypingRunner.java](WtypingRunner.java)（normal/`boom` 两路径，逐成员 try/catch 报告抑制链）。

原冻结 class / 固定 JADX dev（`jadx-cli/build/install/jadx/bin/jadx`）/ Jarde `class-source` 三腿各自 `javac --release 8` 重编后 `java -Xverify:all`，`results/three-way/{run,leg-source}-sha256.txt`：

| 类 | 路径 | 原 class | Jarde | JADX |
| --- | --- | --- | --- | --- |
| T2（`main` 四方法） | normal | `6d30f106…` | **同 SHA** | **同 SHA** |
| Wtyping（六成员经 Runner） | normal | `d0436700…` | **同 SHA** | **同 SHA** |
| Wtyping | boom（体+close 双抛，抑制链） | `53a6ec5c…` | **同 SHA** | **同 SHA** |

Jarde 腿编译披露：T2 按呈现**原样**编译；Wtyping 的 null 对照成员按上节机械调整后编译（五非 null 成员原样可编）。JADX 腿照 17b 惯例保留 `package defpackage;` 以 FQCN 运行；其 saved-return 被自身重构折叠为 `return "in";`（参照列不作语义正例，巡查惯例）。

## 测试与门禁（tasks 2.2/3.1）

- `tests/p3_twr_saved_return_typing.rs`（新增）：T2 命中文本钉死、Wtyping 六变体声明钉死（null 逐字钉 Object）、T2 恢复类**按呈现** `javac --release 8` + `java -Xverify:all` 双腿逐路径一致。
- `tests/p3_twr_discarded_call.rs`（17a，随片更新并注明缘由）：本切片目标正是该文件披露的临时补丁（`compile_ready` 把 `Object local1 = "in"/"solo"` 机械改写为 `return "X";`）所对应的缺陷呈现；细化落地后补丁通道删除，恢复文本按原样编译，三处文本钉死随 spec Requirement（细化对所有 TWR saved-return 生效，非 T2 专属）更新为 `java.lang.String local1 = …`。**此为"既有期望依赖 Object 拼写的非 null 场景"，属本切片目标自身（巡查账本登记的本缺陷），已如实上报 root 复核**；null 期望与 finally 家族期望未动。
- 全仓：`cargo test --workspace --tests --locked --no-fail-fast` 267 目标 0 失败、**2726 passed**（主线 2723 + 新增 3）；`cargo fmt --all -- --check` 通过；CI 同款 clippy（`--workspace --all-targets --all-features --locked` + 存量 allow 清单）零新增；`openspec validate --all --strict` 226 项通过。
- 预算/取消原子性：本片未触预算维度（`constant_of_value` 只读既有 SSA/Operations，无新计费），既有 bulk/cancel 套件全绿为证。

## 已知边界（如实记录）

- 细化只认 `Push(String)`/`Push(Class)` 直推与一层 `Store` 步进；`Load`/`Duplicate` 间接链上的常量（如 concat 中间值）不细化，回退既有拼写——按需扩展，无当前验收需要。
- `MethodType`/`MethodHandle` 常量的 `ldc` 在 facts 层即 `Operation::Other`（decode.rs 明确不命名），无细化分支，保持 `Object`。
- 跨块/phi 保存值（spec 回退场景）由回退通道覆盖（null 对照同通道实证），未单独冻结 phi fixture。
