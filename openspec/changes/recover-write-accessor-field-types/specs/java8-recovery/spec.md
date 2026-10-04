## ADDED Requirements

### Requirement: 返回值形写访问器 SHALL 按字段描述符恢复（封闭类型集）

当合成写访问器（`access$NNN`）的方法体为 `aload_0; <load_1>; <dup 复制>; putfield; <return>`——即"内部类写外部私有字段"的真 javac 产物形——系统 SHALL 在字段描述符属于**封闭类型集**（`Z`/`I`/`B`/`S`/`C`/`F`/`J`/`D`/引用型）时把它恢复为 `arg0.<field> = arg1; return arg1;`，与 boolean 现状同形。每型的装载/复制/返回 opcode 与槽宽 SHALL 以该封闭表为准（单槽：`dup_x1`；`J`/`D` 双槽：`dup2_x1`；下限 stack/locals 相应为 3/2 与 5/3），**不得**外推表外类型。

处理器的**结构判据**（单 SSA 块、无异常表、无 clone 块、BCI 布局 `[0,1,2,3,6]`、`Operation` 序列含 `Load{slot:0}`/`Load{slot:1}`/`Other`/`Field{Write,!static}`/`Return`、字段属于本类、预算语义）SHALL **逐字保留**——本能力只把"类型事实"从 boolean 常量改为查表，不放宽任何结构证明。

static 字段的写访问器与无复制形的返回值写访问器 SHALL 维持既有拒绝（后者属 `accessor@1` 折叠路径的既有语义）。

#### Scenario: 9 类型全部恢复且整类可编译

- **WHEN** 真 javac 8 编译的 `WA` 族（内部类 9 个 setter 各写外部一个不同类型私有字段：boolean/int/long/double/String/byte/short/char/float）经 `class-source` 呈现
- **THEN** 9 个 `access$NNN` 全部恢复为 `arg0.<field> = arg1; return arg1;`；渲染源集 `javac --release 8` exit 0（修复前为 exit 1 + 8 个"缺少返回语句"）；`java` 运行输出与原 class 逐行一致

#### Scenario: boolean 先例零回退

- **WHEN** `PrivateFieldFamily`（`d09f5dea` 的 boolean fixture）与既有行使 `BooleanAccessorAssignment` 的测试经呈现
- **THEN** 渲染文本与修改前逐字节相同；`p3_accessor_edges` 家族断言全部通过

#### Scenario: static 与无复制形仍拒绝

- **WHEN** 写访问器作用于 static 字段，或方法体为无复制形的返回值写（不经 `dup`）
- **THEN** 保持既有拒绝——本能力不放宽字段归属与折叠路径的结构证明

#### Scenario: 结构判据零放宽

- **WHEN** 方法体破坏任一结构判据（多 SSA 块、带异常表、BCI 偏移不同、`putfield` 写他类字段）
- **THEN** 仍拒绝——泛化只覆盖类型事实，结构证明与修改前逐字一致

#### Scenario: 封闭类型集不外推

- **WHEN** 字段描述符不在封闭表内（合成探针）
- **THEN** 维持现状拒绝；新类型入表须先有 javap 实测取证，不允许实现期外推
