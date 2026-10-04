# Design：异构数组初始化器的赋值兼容判据

## Context（root 已实测）

字节码事实（javap）：`Arrays.asList(1, 2L)` → `anewarray java/lang/Number` + `Integer.valueOf(1)`/`Long.valueOf(2L)` 两个 `aastore`。组件类型 `Number` 是 reader 层事实；元素呈现类型由各 `valueOf` 调用描述符给出。现状判据要求"元素呈现类型 == 组件类型"→ Integer ≠ Number → 拒。

## 决策 1：判据改为单向赋值兼容（子→父）

接受的充要条件：**元素呈现类型可赋值给组件类型**（同型、或组件是元素类型的超类/超接口——按既有 `platform_reference_argument_widens` 同源的类层次事实源）。呈现：元素按自身类型呈现（`Integer.valueOf(1)`），数组按组件类型（`Number[]`）——这正是一切合法 Java 数组初始化的语义。

**降向赋值（父→子）与无关节类型仍拒**——判据从"相等"改为"赋值兼容"是**更准确**而非放宽：JLS 数组初始化器本来就要求赋值兼容。

## 决策 2：不引入 LUB/推断

组件类型不做任何计算（来自 `anewarray`/`aastore` 的字节码事实）；元素类型不做推断（来自元素构造调用的描述符）。两事实源均已存在，本片只改一致性检查的比较方向。

## 决策 3：验收与零回退

- 主锚 `CT.cov` 恢复、整类 `javac --release 8` exit 0、`main` 输出 `X/Y/…/1/3` 与原 class 一致（按 fixture 实际输出核对）；
- 零回退：`up()`（同构 String 数组）逐字节不变；`io`/`io2`（instanceof/cast）不变；
- 负例：元素类型**不能**赋值给组件的合成探针（如 `anewarray String` 放 `Integer` store——手工字节码或混淆产物）仍拒。

## 验证标准（可证伪）

1. 主锚恢复 + 行为一致（fixture 双腿：javac23 `--release 8` 与真 javac 8）；
2. 零回退边界如上；负例保持拒绝；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + fingerprint 再生；corpus 双腿扫描差异类仅为异构初始化形。

## Open Questions

1. 拒绝发出处锚点与"组件类型"事实的现有载体（task 1.1 定位）；
2. 呈现具体形（显式 `new Number[]{…}` vs 既有内联形）按仓库数组呈现约定，实现者定并测试钉死。
