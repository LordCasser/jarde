## ADDED Requirements

### Requirement: dup-store 舞蹈 SHALL 按读者数双分支呈现

当一次求值后跟 `dup; istore x` 且 dup 值的另一份被恰一个消费方（条件/返回等）读取时，系统 SHALL 按 store 目标的后续读者数呈现：**零后续读者**（不可观察赋值）消除 store、表达式直接进消费位（与 jadx 同构）；**有后续读者**拆为 `x = <expr>;` 语句在前、消费位读 `x`。

dup 值存在**多于两个**消费方（store + 条件之外仍有栈残留读者）时 SHALL 保持既有拒绝。copy 值家族其它三形状（数组 dance、后缀旧值、putfield 链）的既有行为 SHALL 不变。

#### Scenario: 不可观察赋值消除

- **WHEN** `static boolean condAssign(int x){ return (x = x + 1) > 0; }`（参数槽赋值无后续读者）经 `class-source` 呈现
- **THEN** 恢复（如 `return arg0 + 1 > 0;` 同构形）、0 引注；整类渲染源集 `javac --release 8` exit 0、行为与原 class 逐行一致

#### Scenario: 有读者拆语句

- **WHEN** 赋值目标在条件之后仍被读取（局部变量形）经呈现
- **THEN** 拆为先赋值语句再条件（`local0 = …; return local0 > 0;` 同构）、行为一致

#### Scenario: 家族零回退

- **WHEN** 移位复合、NaN/Inf 除法编码、家族前三员既有测试经运行
- **THEN** 逐字通过

#### Scenario: 多读者仍拒绝

- **WHEN** dup 值在 store+条件外另有读者（负例）
- **THEN** 保持既有拒绝
