# Design：int 化布尔操作数的位运算上下文回投

## Context（root 已实测）

- 拒绝点：位运算两操作数呈现类型分别为 `int` 与 `boolean` → "no Java integer operator" 拒。
- 机制：javac 把布尔值 int 化——`!b` 编为 `load b; iconst_1; ixor`（产 int 0/1）；`r ^= x` 的累积变量编为 int 计数器（出口 `% 2 != 0` 既有域已恢复）。
- jadx 有解（boolean 累积 / `z & (!z2)`）；同型全恢复——判别完整。

## 决策 1：回投条件 = 生产链全在布尔位运算上下文（保守）

一个 int 操作数可回投为 boolean 当且仅当其**每一条生产链**满足：值来源是布尔（boolean load 经 `!`/条件产生的 0/1、或 boolean→int 的显式转换点）**且**其**全部消费**都在位运算内或既有 boolean-from-int 出口。任何非布尔消费（算术、比较、int 存储、方法实参）→ 不回投、维持拒绝。这是数据流充分性判据，不是语法猜测量。

## 决策 2：呈现 = 直接布尔形（与 jadx 同构）

`a & !b` → `arg0 & !arg1`；`r ^= x` 循环累积 → 循环变量呈现为 boolean（`boolean local = false; … local ^= x;`），出口直接 `return local;`（既有 `% 2 != 0` 出口在变量回投后自然消解——若出口呈现仍需该形，实现者按数据流如实选择并在测试钉死）。

## 决策 3：零回退与负例

- 同型（boolean^boolean / int^int / 移位族 / Kernighan）逐字节不变；
- 负例：int 与 boolean 的**真混合算术**（如 `(a?1:0) + b` 存 int）仍拒；
- corpus 双腿扫描：差异类仅为混合布尔位运算形（语料预期 0，如实记录）。

## 验证标准（可证伪）

1. 主锚：`BW.andNot`/`BW.mix` 恢复、整类 `javac --release 8` exit 0、`main` 输出逐行一致（`true/5/true/2/-2147483648/false/3` 按实际）；
2. 零回退与负例如上；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 拒绝发出处的精确落点与操作数类型呈现的现有载体（task 1.1 插桩）；
2. 回投判据的实现层次（呈现层操作数适配 vs 布尔变量识别提前）——按落点实际选最小改动，报告说明。
