## ADDED Requirements

### Requirement: Proved enum constants preserve bounded integer constructor arguments

完整 Java 8 enum 类源码投影 SHALL 保留每个 enum 常量的已证明 int 构造实参，包括整数 literal、一次 `int` 静态字段读取，以及该读取与一个整数 literal 的直接加法表达式。静态字段的属主、字段名、类型、实参顺序及求值位置 SHALL 与同次类源码运行读取的物理引用和指令证据一致；系统 MUST NOT 把静态字段读折叠成常量或改变其求值次数。该实参 grammar MUST 拒绝方法调用、存储、分支、额外或重复字段读取及未声明支持的算术。

静态字段叶 SHALL 仅在选定环境中唯一解析到完整 Java 8 顶层、同包可访问的 class facts，并在唯一匹配的字段表项上证明 descriptor 为 `I`、`static`、非 `final` 且非 synthetic 时进入投影。系统 MUST 拒绝同枚举 owner、跨包 owner、含 `$` 的二进制 owner、private/protected、synthetic 或 interface/annotation owner、带 `EnclosingMethod` 或声明自身为 member 的 owner class attributes、字段缺失或歧义，以及依赖读取不完整的情况。source spelling SHALL 使用同包短类型名；若 enum 的完整 `InnerClasses` facts 存在遮蔽该短名的 member type，则 MUST 拒绝。预算或取消停止必须保持为 stopped，不得转为正常 refusal 或部分投影。

#### Scenario: One integer static field and field-plus-literal arguments
- **WHEN** 一个 Java 8 enum 的完整常量初始化前缀在 constructor call 处依次传递 `Ints.THREE` 与 `Ints.THREE + 1`，且所有表达式都由该次 class-source 读取中的精确字段引用、int literal 和加法指令证明
- **THEN** 完整 enum 声明 SHALL 输出 `FIELD(Ints.THREE)` 和 `EXPR(Ints.THREE + 1)`，并且重编译后的常量值、顺序和可观察初始化行为 SHALL 与原 class 一致

#### Scenario: Existing literal arguments remain supported
- **WHEN** 所有 enum 常量都以现有已证明的 int literal grammar 调用唯一受支持的 enum constructor，且其余整组证明完整
- **THEN** class-source SHALL 继续将完整有序常量组投影为 enum constants，输出的 literal 值和构造器字段行为 SHALL 与原 class 一致

#### Scenario: Unsupported integer argument rejects the whole group
- **WHEN** 任一常量参数包含方法调用、存储、分支、多次静态读取、未支持算术，或不能绑定到唯一 constructor call 和字段写
- **THEN** 系统 MUST NOT 投影该 enum 的部分常量组；它 SHALL 保留现有 fallback/markers 和可审计的物理成员事实

### Requirement: Enum constant projection remains atomic across the full proof

受限 int 实参 SHALL 仅在完整常量组证书成立时进入 Java enum 声明。该证书 MUST 覆盖物理常量字段和 field-table 次序、每次初始化构造调用与目标字段写入、初始化序列剩余部分、值数组工厂、`values()`、`valueOf()`、constructor 行为以及隐藏成员 use census。预算、取消或任一不完整/歧义证据 MUST NOT 发布部分 enum projection；未证明的 initializer suffix 和其他成员效果 MUST 保留或拒绝。

#### Scenario: One constant argument cannot be proved
- **WHEN** 一个多常量 enum 只有部分实参符合受限 grammar，或一个 constructor call、目标字段写、数组发布/use census 证据不完整
- **THEN** 系统 MUST 拒绝整组 enum constant projection，MUST NOT 把已通过的常量从普通字段中单独挑出

#### Scenario: Static field class initialization remains observable
- **WHEN** 一个已证明的实参通过静态字段读取触发其属主类的初始化，且读取前后存在可观察的初始化效果
- **THEN** 输出 SHALL 在对应常量构造实参位置保留该静态字段读取一次，不得预先求值、搬动求值位置或将其替换成缓存 literal

#### Scenario: Static field source metadata is not safe to emit
- **WHEN** 字段为 `final`、不可访问、字段目标缺失或重复、owner 位于其他 package/自身 enum/未证明的 binary member name，或所选环境不能唯一解析 owner
- **THEN** 完整 enum 常量组 MUST 整体拒绝；系统 MUST 保留普通物理字段及 initializer fallback，不得猜测 owner 或字段值

#### Scenario: Dependency read stops
- **WHEN** 选定环境对外部字段 owner 的解析或完整字段表读取因预算或取消停止
- **THEN** enum proof SHALL 保持 stopped，且 MUST NOT 发射任何该组常量

enum initializer prefix MUST 由同次完整、无 exception handler 的 raw Code/BCI 证书逐指令证明，包括每个常量的构造、实参求值、constructor call、`putstatic` 顺序与符号引用，以及 `$VALUES` 工厂和发布。AST sidecar 仅提供同次 initializer 身份与 handler 状态，不得因其无法分类交错 `new; dup; getstatic; invokespecial` 而否定完整 raw Code 证书。物理 initializer method fallback、markers 和诊断 MUST 保留。prefix 后仅可接受已证明的单条 terminal return 或当前既有完整 static assignment suffix；其他尾部 MUST 拒绝投影。

## Non-Goals

本 change 不证明或发射 `String...`/数组参数，不支持其他算术或方法调用，也不更改单方法 recovery 或任意 constructor body 的通用表达式范围。`TestEnums4` 风格的空 varargs、显式数组 literal 与 Signature 尾参由独立后续任务定义。
