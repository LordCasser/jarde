## Why

[写访问器巡查](../../evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/README.md)实测：内部类写外部**私有字段**时，真 javac（8 与 23 皆然，版本无关）生成**返回值形**写访问器 `access$NNN`（`aload_0; <load_1>; dup_x1; putfield; <return>`）。jarde 已有一条**已验证的处理器**（`BooleanAccessorAssignment`，`crates/jarde-java/src/build.rs:8755` 起，由已合入的 `d09f5dea` 交付），但它把字段类型**硬编码为 `boolean`**（描述符 `(L{owner};Z)Z`、opcode 表、`field.descriptor == "Z"`）。

9 类型全量测量（[results3/nine-type-measurement.txt](../../evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/results3/nine-type-measurement.txt)）：**9 个类型中 8 个被拒**——`int`/`long`/`double`/`float`/引用型/`byte`/`short`/`char` 全部渲染为空 stub 而保留返回类型 → 渲染源集 `javac --release 8` **exit 1、8 个错误全部 "缺少返回语句"** → **整类不可编译**。这是常见形（内部类写外部 `int`/引用私有字段极常见），影响是"整类不可编译"级别的正确性缺口，非呈现润色。

判别变量**只有字段描述符**：boolean 与 int 的 opcode 序列、BCI 布局、stack/locals **逐字相同**（巡查第一节之二的决定性对照），唯一差异是 `field.descriptor` 的 `Z` vs `I`。

## What Changes

把 `BooleanAccessorAssignment` 从"仅 boolean"泛化为"封闭的 9 类型集合"，按描述符查每型事实表：

- **单槽值型**（`int`/`byte`/`short`/`char`）：与 boolean 完全同 opcode（`iload_1`+`dup_x1`+`ireturn`）、同 BCI、同 stack/locals——只需放宽描述符门。
- **`float`**：`fload_1` + `freturn`（复制仍是 `dup_x1`）。
- **引用型**：`aload_1` + `areturn`（复制仍是 `dup_x1`）。
- **`long`/`double`**（双槽）：`lload_1`/`dload_1` + **`dup2_x1`** + `lreturn`/`dreturn`，`max_stack=5`/`max_locals=3`（下限门须按槽型放宽）。

呈现不变：`arg0.<field> = arg1; return arg1;`（boolean 现状即此形）。**不新增机制**：判据仍由描述符唯一决定，处理器结构、`fields.claim` 所有权、预算语义全部不动。

## Impact

- **代码**：仅 `crates/jarde-java/src/build.rs` 的 `BooleanAccessorAssignment`（更名或保留名+扩判据由实现片定，倾向改名为反映"写访问器赋值"的名字并让 boolean 成为表中一行）；`LongAssignmentResult` 消费链若因 `dup2_x1` 双槽需要区分，在既有结构内处理。**不触碰** `accessor.rs`（那是另一条 `accessor@1` 折叠路径，其 `returns.is_some()` 拒绝与本文无关）。
- **测试**：新增 9 类型正例（冻结 fixture 复用巡查的 `WA` 族，真 javac 8）+ boolean 零回退断言（`d09f5dea` 的既有 fixture `PrivateFieldFamily` 逐字节不变）。
- **账本**：EM-15 缺口登记（summary.md）从"待独立立项"改为指向本片；落地后由 root 验收时更新。
- **验收锚纪律**：主锚 = `WA` 族 9 类型（`int`/引用型是最常见形，必须有）；零回退锚 = `PrivateFieldFamily`（boolean 先例）逐字节不变。

## Non-Goals

- **不**改 `accessor.rs` 的 `accessor@1` 折叠路径（`FieldAccess::Write` 分支的 `returns.is_some()` 拒绝保持——那是"未带 `dup` 复制形"的正确拒绝，本片只处理带复制的返回值形）。
- **不**支持复合写形（`i += 1` 生成的读-改-写访问器）——那有不同的 opcode 结构，须另行取证。
- **不**支持 static 字段的写访问器（现有处理器明确 `!field.is_static`，真 javac 对 static 私有字段写不生成 access$ 实例形）。
- **不**做全语料 `access$` 普查的补救重编（语料本身无真实 `access$` 产物，是已登记的盲区事实，不在本片解决）。
