## ADDED Requirements

### Requirement: 非循环标签块的 break SHALL 呈现为 if 嵌套等价形

当 `break` 的目标是**非循环标签块**（标签标记的语句块而非循环）且其跳转目标（标签块结束的 join 点）在方法几何内可证时，系统 SHALL 把该标签块呈现为 if 嵌套等价形——每个 break 的效果表达为"条件不满足时跳过该标签块的剩余语句"（与 jadx 呈现同构，与 finally-return 的等价退化同族）。

跳转目标**不可证**（深嵌套与异常边交错等）时 SHALL 保持既有拒绝。循环标签（`break loopN`）的既有通道与呈现 SHALL 逐字不变；无限循环的 do-while 归一化呈现 SHALL 逐字不变。

#### Scenario: 双层标签块恢复

- **WHEN** `outer: { inner: { if(x==1) break outer; if(x==2) break inner; s+=1; } s+=10; } s+=100;` 经 `class-source` 呈现
- **THEN** 恢复为 if 嵌套等价形（0 引注）；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致（x=1→100、x=2→110、x=3→111）

#### Scenario: 循环标签零回退

- **WHEN** 循环标签 break（`break loopN`，既有测试族）经呈现
- **THEN** 既有呈现逐字不变——两通道互斥判定

#### Scenario: 不可证 join 仍拒绝

- **WHEN** break 目标的 join 几何不可证（负例探针）
- **THEN** 保持既有拒绝——呈现由可证性驱动，非语法猜测
