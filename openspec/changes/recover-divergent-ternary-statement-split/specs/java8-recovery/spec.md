## ADDED Requirements

### Requirement: 异型分支条件表达式的 return 消费 SHALL 语句化呈现

当条件表达式的两分支值呈现类型异构（本呈现层无法证明公共条件类型）**且**其消费点是方法体内单一 `return` 语句时，系统 SHALL 把该语句呈现为 if/return 语句拆分形（`if (cond) { return a; }` + 后继 `return b;`，与既有 if/else 呈现约定一致），而非整方法拒绝。

其它消费形态（赋值、方法实参、嵌套子表达式）SHALL 保持既有拒绝（"two values joined … do not have a conditional Java type"）——语句化不适用于需要临时变量与求值顺序约束的场景。同型分支（含 int 链、嵌套、引用同型）的条件表达式呈现 SHALL 逐字不变。

#### Scenario: return 消费形恢复

- **WHEN** `static Object poly(boolean c){ return c ? Integer.valueOf(1) : "s"; }` 经 `class-source` 呈现
- **THEN** 方法体呈现为 if/return 拆分形（无整方法拒绝、无引注）；整类渲染源集 `javac --release 8` exit 0、运行输出与原 class 逐行一致

#### Scenario: 实参嵌套形保持拒绝

- **WHEN** 异型分支三元作为方法实参（`foo(c ? Integer.valueOf(1) : "s")`）且无公共可证类型
- **THEN** 保持既有响亮拒绝——语句化能力不外推到需要临时变量的消费形

#### Scenario: 同型分支零回退

- **WHEN** 同型分支条件表达式（int 链、嵌套三元、引用同型）经呈现
- **THEN** 既有三元呈现逐字不变
