## 1. 同一 case 的续接证明

- [x] 1.1 核对 `switch_region` 丢弃子 region `Run.next` 的实际路径，限定在受当前 case 支配、无外部 predecessor、未越过其它 case/父 boundary 的后继，继续有界 Region 走访。
- [x] 1.2 多入口、cross-case、循环/异常边和多个续接候选均保持保守拒绝；预算/取消不留下部分发布。

## 2. 结构和 String 证书

- [x] 2.1 使内层 hash 与 final switch 在同一个 outer default arm 的有序 `Region::Sequence` 中，复用现有 `project_string_switches` 和 `stringswitch::prove`；不在声明规划处绕开 slot 4 fallback。
- [x] 2.2 检查 BCI 62/71/96/105/111/120/123/125/152/154/156 的唯一 owner 和来源，`choose(String)` 无 `@bytecode`，普通碰撞与独立 hash 用途负例不退化。

## 3. Java 8 对照验收

- [x] 3.1 用 CF-14 固定输入重编原/JADX/Jarde 完整类，嵌套样本五行、普通样本八行分别一致；独立 hash 用途原/Jarde 六行一致，保留固定 JADX `r0` 编译失败事实。
- [x] 3.2 运行定向 String/switch/Region 测试、格式、适用 crate check、`git diff --check` 与 `openspec validate recover-nested-string-switch-arm-continuation --strict`，记录验收并清理独立 Cargo target。
