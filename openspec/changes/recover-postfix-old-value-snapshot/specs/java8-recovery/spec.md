## ADDED Requirements

### Requirement: 后缀自增的旧值快照 SHALL 呈现为后缀表达式

当局部变量的自增序列为 `iload x`（旧值）先于 `iinc x` 入栈、且该旧值**恰好有一个**消费方（赋值捕获或表达式内消费）时，系统 SHALL 在消费位呈现 `x++` 后缀表达式形——消费方收到旧值语义（`int j = x++;` 使 j 为自增前的值）。

前缀（`++x`，无快照）、拆两语句（`x++; j = x;`）、复合赋值表达式（捕获新值）的既有呈现 SHALL 逐字不变。旧值存在**多个**消费方、或消费目标与自增 slot 同体（`x = x++` 自赋值形）时 SHALL 保持既有拒绝。

#### Scenario: 赋值捕获旧值

- **WHEN** `static int immUse(){ int i = 5; int j = i++; return j; }` 经 `class-source` 呈现
- **THEN** 恢复为后缀形（`int local1 = local0++;` 或同构呈现）、0 引注；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致（j 收到旧值 5）

#### Scenario: 表达式内消费旧值

- **WHEN** `return i++ + 10;`（旧值参与算术）经呈现
- **THEN** 恢复（`return local0++ + 10;` 或同构）、行为一致

#### Scenario: 健康形零回退

- **WHEN** 前缀 `++x`、拆语句 `x++; j = x;`、复合赋值捕获（`y = (x += 5)` 捕获新值）经呈现
- **THEN** 既有呈现逐字不变

#### Scenario: 自赋值与多消费方仍拒绝

- **WHEN** `x = x++;`（旧值存储回同 slot）或旧值被两处消费
- **THEN** 保持既有拒绝
