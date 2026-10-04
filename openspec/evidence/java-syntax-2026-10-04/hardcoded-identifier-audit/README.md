# 生产码硬编码标识符字面量普查（2026-10-04，root）——缺陷类审计

由 [enum-string-field-name-hardcode](../enum-string-field-name-hardcode/README.md) 的缺陷模式（"已验收能力只在自身 fixture 的标识符上生效"）触发，root 对全部生产码做了一次同类审计，判断是否存在其它"验收锚恰为满足硬编码而设计"的自我实现式验收。

**方法**：grep 生产码（排除各文件的 `#[cfg(test)]` 模块起点之后）中 `== b"<标识符>"` 形式的名字比较，逐个判定其性质。**关键区分**：
- **语言/JDK 强制的名字** → 正确（不是缺陷）。
- **用户自定义名被硬编码，且 spec 声称通用能力** → **缺陷**（实现窄于自身 spec，验收自我实现）。
- **用户自定义名被硬编码，但代码/spec 明确记载为"窄首片"** → 可接受的 MVP 边界（非缺陷，但登记为已知限制）。

## 审计结果

| 落点 | 硬编码名 | 性质 | 判定 |
| --- | --- | --- | --- |
| `facade.rs:2653` | `value`（`()F` 注解元素） | JLS 规定的单元素注解默认名 | **正确**（语言强制） |
| `lambda.rs:1045` | `intValue`（`java/lang/Integer`） | JDK `Integer.intValue()` | **正确**（JDK 强制） |
| `facade.rs:14094/15906/15920/15923/15928` | `op`（String 实参枚举字段） | 用户字段名，且发射文本字面写死 `this.op` | **缺陷** → [enum-string-field-name-hardcode](../enum-string-field-name-hardcode/README.md)；其 spec Requirement 声称"一个可无损拼写的 ASCII 字符串实参"、无字段名条件，实现窄于 spec；已立 [recover-enum-string-field-name-generalization](../../../changes/recover-enum-string-field-name-generalization/) |
| `enum_constants.rs:711/785`、`class_source.rs:8309` | `totalUnits`（static int 字段）、`sumUnits`（`()I` static 方法） | 用户自定义名，源自证据 fixture `Measure.java`（`java-syntax-2026-09-22/enum-declaration/user-static-boundary/`） | **可接受的窄首片**（见下） |

## `totalUnits`/`sumUnits`：文档记载的窄首片，非缺陷

宿主函数 `prove_static_assignment_suffix`（`enum_constants.rs:655`）的 doc 注释**明确记载**其窄性：

> "Prove the **one static assignment suffix frozen by the Measure/Counted fixtures**. The fixed source constants and BCI shape **deliberately keep this first slice narrow**. A different user suffix remains in the original `<clinit>` presentation, along with every physical enum member; it is never trimmed from emitted text."

关键区别：(1) 该函数是**证明一条特定的静态赋值后缀**，其 narrow 边界写在代码里，不像 `op` 那样在 spec 里声称通用却在实现里偷偷收窄；(2) 引入提交 `2ad29cee` 的 change 未把"任意用户静态字段名"列为已交付能力；(3) root 实测 `grep -rln "prove_static_assignment_suffix\|Measure/Counted" openspec/specs/` 无命中——即主规格**没有**声称通用能力，故不存在"实现窄于自身 spec"的落差。

**故不立修复片**，但登记为**已知限制**：`prove_static_assignment_suffix` 只对字段名 `totalUnits` + 方法名 `sumUnits` 的 `Measure/Counted` 形生效，其它用户静态后缀形按其 doc 注释保持原 `<clinit>` 物理呈现（"never trimmed from emitted text"，即不产出半投影——**此为该代码自述契约，root 本次为零构建审计、未实测非 `totalUnits` fixture**，与 `op` 案不同，`op` 案 root 已实测六形对照）。若将来要泛化，须像 `op` 修复那样把已证名从字节码通路传到发射处，而非硬编码——且**须补一个非 `totalUnits` 名的冻结正例**，否则重蹈 `op` 的自我实现式验收覆辙。

### 登记为独立债务（root 2026-10-04 复核后追加）

**该能力的"窄性"目前只记载在代码注释里，没有任何 spec 或账本行承载它。** root 复核：

| 承载处 | 是否记载名字依赖 |
| --- | --- |
| `enum_constants.rs:655` 的 doc 注释（"frozen by the Measure/Counted fixtures … deliberately keep this first slice narrow"） | **是**（唯一记载处） |
| OpenSpec：`grep -rln "prove_static_assignment_suffix\|static_assignment_suffix\|StaticAssignment" openspec/changes/ openspec/specs/` | **无命中**（除本片自己的 design 提及） |
| inventory 账本：`grep -rln "Measure\|Counted\|静态赋值后缀\|static assignment" openspec/evidence/jadx-feature-inventory-2026-09-27/*.md` | **无命中**——无任何 DT/CF/EM 单元承载"枚举用户静态初始化后缀"这一能力 |
| 其自身证据 [user-static-boundary/README.md](../../java-syntax-2026-09-22/enum-declaration/user-static-boundary/README.md) | **未记载名字依赖**——它逐处描述该 fixture（"Jarde already preserves the user `totalUnits` field, the body of `sumUnits()`"），但**没有**说明这些名字是被硬编码要求的，故读者会误以为能力对任意用户静态字段名成立 |

引入提交为 `2ad29cee`（"feat: expand Java syntax recovery and differential evidence"，一次批量提交，未附带独立 change）。

**风险与 `op` 同型**：能力边界只活在代码注释里，一旦该函数被重构或注释丢失，窄性就无人知晓；而账本与证据都不记载，未来巡查者可能把"枚举用户静态初始化已恢复"当成既成事实，或反过来重复实现一遍。**处置**：不立修复片（窄首片可接受；其回退按代码自述契约保持物理 `<clinit>` 呈现、不产出半投影，故非静默偏离——但如上所述 root 未实测该形），但**须补两处记载**——(1) 在其证据 README 追加名字依赖说明（**已于 2026-10-04 完成**，见 [user-static-boundary/README.md](../../java-syntax-2026-09-22/enum-declaration/user-static-boundary/README.md) 的 "Known limitation" 段）；(2) 在 inventory 账本相应单元（枚举域 DT-10 或新开单元）登记"仅覆盖 `Measure/Counted` 形，其它用户静态后缀形保持 `<clinit>` 呈现"——**此项仍未做**（DT-10 现记载为"空枚举、普通枚举常量"，未涉用户静态初始化后缀；该能力当前无任何账本单元承载，见上表）。这两项属文档债，(2) 可并入 `recover-fixture-behavior-guard-coverage` 或单独一次 docs 提交完成。

## 系统性教训（本审计的价值）

**"验收锚恰为满足实现的硬编码而设计"是一类隐蔽缺陷**：CI 全绿、spec 通过、原/JADX/Jarde 三方对照一致，但能力只在锚的特定标识符上成立。它无法被"锚通过"发现，只能被**变更一个正交变量（此处是字段名）的对照探针**发现。root 本次正是用六形对照（仅字段名不同、结构逐字节同构）钉死了判别变量。

**防范措施**（写进 handoff 的验收门禁口径）：任何"证明一条特定用户标识符形态"的切片，其验收必须包含**至少一个标识符名与锚不同的正例**（若实现正确泛化，它应同样通过；若实现硬编码了锚名，它会失败）。这一条能自动暴露 `op` 型缺陷。语言/JDK 强制的名字（`value`/`intValue`/`<init>`/`compareTo` 等）不在此列——它们本就该硬编码。
