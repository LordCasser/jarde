# Design：非循环标签块的 if 嵌套等价呈现

## Context（root 已实测）

- 拒绝点："the intermediate join, bridge and outer conditional do not form a closed value"——break 跳过中间 join 时值流不闭合。
- jadx 解法：`break outer` → 外层 if 包住 inner 之后的语句；`break inner` → 内层 if 包住 inner 剩余。嵌套深度=标签深度。
- 行为等价验证：jadx 形 stub 后输出逐行一致（100/110/111）。
- 循环标签（`break loopN`）走既有通道恢复——非循环标签是另一落点。

## 决策 1：等价形 = if 嵌套（与 jadx 同构）

每个 `break L`（L 为非循环标签）呈现在其条件处的效果是：**L 块中该 break 之后的所有语句被跳过**。用 if 嵌套表达：内层块尾部语句进 `if (!(break 条件))`，逐层外推。呈现由数据流可证性驱动（跳过可证才呈现），非语法猜测。

## 决策 2：判据 = 跳转目标可证（join 几何）

接受条件：break 的跳转目标（标签块结束的 join 点）在该方法的几何内**可证**（既有 region/join 事实）；不可证保持拒绝。与循环标签通道互斥判定（目标是否循环头）。

## 决策 3：零回退与负例

- 循环标签（`break loopN`）既有呈现逐字不变；
- 无限循环 do-while 归一化（同巡查已证）逐字不变；
- 负例：break 目标 join 不可证形（深嵌套+异常边交错——如实构造）保持拒绝；
- corpus 双腿扫描：预期 diff 为空（语料可能 0 非循环标签，如实记录）。

## 验证标准（可证伪）

1. 主锚：`LB.labeledBlock` 恢复为 if 嵌套等价形（0 引注）、整类 `javac --release 8` exit 0、`main` 输出 `100/110/111/6/5/104` 逐行一致；
2. 零回退/负例如上；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 拒绝发出处与 join/桥事实的现有载体（task 1.1 插桩）；
2. 三层以上嵌套标签（嵌套 if 深度增长）——MVP 先两层（fixture 形），三层按实测扩验（如实记录是否触发呈现退化）。
