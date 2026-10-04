## Why

[DT-26 已证差距](../../evidence/jadx-feature-inventory-2026-09-27/declarations-types.md)（root 双 javac 扫描登记）：lambda 捕获**原生数组**局部（`int[] t={0}; l.forEach(i -> t[0]+=i)`）时整个 invokedynamic 站点被拒，落点 `crates/jarde-java/src/lambda.rs:769`（`jre_lambda_sam_types`）：捕获操作数在 SSA 帧中是 `Object` 而在站点描述符与实现中是 `int[]`。**同探针内引用类型捕获（`StringBuilder s; l.forEach(x -> s.append(x))`）内联正常**——判别变量是**捕获值的类型来源**。

**根因（root 零构建读码定位，frame.rs:2299-2315）**：帧层对 `newarray`（原生数组）**有意**压入 `Value::Ref(RefType::Unknown)`——注释原文"The array of a primitive element type is defined by the bootstrap loader, which this request does not declare, so the slot holds a conservative unknown reference instead of a name this request cannot anchor"。而 `anewarray`（引用元素）经 `pool_class_name` 得**有名**类型。即：帧只锚定**常量池可命名**的类型，原生数组 `[I` 是描述符而非池类名，故帧层保守 Unknown（呈现为 `Object`）——lambda 捕获门（`lambda.rs:765-779` 三方类型一致判据）把帧 Unknown 与站点/实现的 `int[]` 判为不一致 → 拒。**这是响亮拒绝，非静默偏离**；但 `int[] t; l.forEach(i -> t[0]+=i)` 是真实高频形（forEach 累加、闭包计数器），双腿（真 javac 8 与 javac 23）引注相同，与版本无关。

## What Changes

让捕获门拿到**有据可查**的数组类型，而不是把帧的保守 Unknown 判为不一致。落点二选一（由实现片按 task 1.1 插桩定夺，见 design 决策 1）：

- **方向 A（帧层）**：`newarray` 的结果类型从 `RefType::Unknown` 升级为**有名**的原生数组类型（按 `atype` 码映射 `[Z`..`[D` 十种）——但须先回答帧层当初保守的**理由是否仍成立**（"bootstrap loader 未声明"——若帧的类型格本就不需要池锚定即可命名描述符形，则保守是历史性的；若其它消费者依赖 Unknown 的保守性，则 A 会波及面大）。
- **方向 B（捕获门）**：捕获操作数若来自**局部装载**且该局部的 LVT/名字表陈述了类型（`[I`），则以陈述类型参与三方一致判据（帧 Unknown 不构成反证）——与 `nested_member_reference_spelling`"证据不能发明，但陈述的事实可用"同构；不动帧层。

呈现：`l.forEach(i -> t[0] += i)` 按 DT-26 既有内联形（与 `map` 的 `StringBuilder` 捕获同构）。**不放宽**三方一致判据本身（`frame_type != site_type || site_type != implementation_type` 逐字保留——只是让 `frame_type` 拿到正确的值）。

## Impact

- **代码**：`crates/jarde-jvm/src/frame.rs`（仅方向 A）或 `crates/jarde-java/src/lambda.rs` 的 `capture_types` 来源（方向 B）+ `names.rs`（若 B 需要读局部陈述类型）。**不触碰** `lambda.rs:765-779` 的一致判据结构、不触碰站点描述符/实现的解析。
- **测试**：`P02_lambda` 冻结 fixture（真 javac 8 + javac 23 双腿）作主锚；`StringBuilder` 捕获（现正常）零回退锚；负例 = 捕获值类型真不一致的形（如站点与实现各说一词的合成探针）仍拒。
- **账本**：DT-26 行的"已证差距……待独立立项"改指本片；落地后由 root 验收更新。

## Non-Goals

- **不**处理 `multianewarray` 捕获（`int[][]` 等）——同族但独立取证后另立（若方向 A 落地其自然覆盖，则验收时实测并在账本记录，不另立项）。
- **不**改 `anewarray` 路径（已正常）。
- **不**做 lambda 体内数组元素复合赋值的表达式化（`t[0]+=i` 在 helper 体内如何呈现是 DT-26 既有域，捕获门放行后按既有路径走）。
- **不**放宽三方一致判据（`jre_lambda_sam_types` 的语义是防静默转换，保留）。
