## ADDED Requirements

### Requirement: 位运算的 int 化布尔操作数 SHALL 按数据流充分性回投

当位运算（`&`、`|`、`^`）的一侧操作数呈现为 `int` 而该值的**每一条生产链**都源于布尔（boolean 经 `!`/条件产生的 0/1、或 boolean 累积计数器）**且其全部消费**都在位运算内或既有 boolean-from-int 出口时，系统 SHALL 把该操作数回投为 boolean 呈现——`a & !b`、`r ^= x`（布尔累积循环）恢复为直接布尔形（与源写法同构）。

int 值存在**任何非布尔消费**（算术、比较、int 存储、方法实参）时 SHALL 保持既有拒绝——回投是数据流充分性判据，不是语法猜测量。同型位运算（boolean 与 boolean、int 与 int、移位族、位技巧循环）的既有呈现 SHALL 逐字不变。

#### Scenario: andNot 恢复

- **WHEN** `static boolean andNot(boolean a, boolean b){ return a & !b; }`（`!b` 编为 int 0/1）经 `class-source` 呈现
- **THEN** 恢复为布尔形（如 `arg0 & !arg1`）、0 引注；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致

#### Scenario: 布尔累积循环恢复

- **WHEN** `r ^= x` 的 boolean 累积（javac 编为 int 计数器 + `% 2 != 0` 出口）经呈现
- **THEN** 循环变量回投为 boolean（`z ^= z2` 形）或既有出口如实保留（实现择一并测试钉死）；行为一致

#### Scenario: 真混合算术仍拒绝

- **WHEN** int 值（布尔来源）存在非布尔消费（如存入 int 局部后参与算术）
- **THEN** 保持既有拒绝——回投不外推到数据流不充分的形

#### Scenario: 同型零回退

- **WHEN** 同型位运算（boolean^boolean、int^int、移位、Kernighan 循环）经呈现
- **THEN** 既有呈现逐字不变
