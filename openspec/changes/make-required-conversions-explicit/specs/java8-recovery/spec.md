## MODIFIED Requirements

### Requirement: Boolean contexts are presented as booleans

值出现在 boolean 上下文时，呈现 MUST 按该上下文的类型定型，而不是按值在帧里的 int 形状拼写：返回类型为 `Z` 的方法的 `return` MUST 以 boolean 呈现（`return true;`/`return false;`，MUST NOT 为 `return 1;`/`return 0;`）；条件的分支测试在其操作数被证明为 boolean（返回 descriptor 为 `Z` 的调用结果，或既有 P3-R5 证据所证明的 boolean 参数、由它声明的局部、字面量与否定）时 MUST 写成真值测试（`if (flag())`、`if (arg0)`），MUST NOT 写成与 `0` 的整数比较（`if (flag() != 0)`）。层不能证明上下文所要求的值是 boolean 时 MUST 拒绝该区域并保留 bytecode 与 origin，MUST NOT 发布另一个编译器拒绝的文本；上下文没有 boolean 要求的位置（真正 `int` 返回、两个 int 值的比较）MUST 保持原有整数形态。本要求只使用返回 descriptor、参数槽类型与既有 callee descriptor 证据，MUST NOT 由它引入通用类型系统或推断。

本要求同样约束判定的**顺序与操作数范围**：某个位置是否要求 boolean MUST 在该位置的**全部**操作数被拼写之前判定，且 MUST 由该位置的形态与已证明的证据作出，而不是由操作数各自的值形态作出。值自身的类型证据（`Z` 参数槽的读取、返回 descriptor 为 `Z` 的调用结果、descriptor 为 `Z` 的字段读取、被本 run 证明 boolean 的局部的读取，以及真正 `int` 操作数的整数形状）只回答「这个值是什么」；目标位置的要求回答「这个位置要不要 boolean」；`0`/`1` 到 `false`/`true` 的拼写只是后者的**适配**，MUST NOT 反过来成为前者的证据。因此整数二元比较（`if_icmp*` 形态）的两个操作数 MUST 保持各自的整数文本：`1 == n`、`n == 1`、`0 < n`、`n > 0` 的常量操作数写成 `1`/`0`，变量操作数按自己的值写出；常量在左、常量在右、相等与大小比较一律相同，MUST NOT 出现 `true == arg0`、`false < arg0`、`true < arg0` 一类 javac 拒绝的文本。**比较的结果是 boolean，这一事实 MUST NOT 被当作它操作数的类型证据。** 操作数被**证明**为 boolean 的零值测试（`ifeq`/`ifne` 形态）仍 MUST 写成真值测试（`if (b)`、`if (!b)`、`if (flag())`），而不是 `if (b != 0)`。同一个 `iconst_1` 在二元比较里 MUST 是 `1`，在真值测试与 `Z` 返回里 MUST 是 `true`。

这条修正是对本项目自己上一轮 boolean 修正（已归档的 `type-boolean-contexts`，`5a8c36a`）所引入回归的纠正：该修正之前（`fa6dc6e`）`1 == n`、`0 < n`、`1 < n` 三个形状输出正确的整数文本；本要求 MUST NOT 被表述为新发现的缺陷，MUST NOT 重开该 change 的其它结论。前一轮已经落地的 boolean 形状与 int 形态 MUST 逐字保留。

#### Scenario: A boolean return presents true or false

- **WHEN** 方法 descriptor 的返回类型是 `Z`，正文是 `if (x == 0) { return true; } return false;`（`iconst_1`/`iconst_0; ireturn`）
- **THEN** 呈现 MUST 为 `return true;`/`return false;`，MUST NOT 为 `return 1;`/`return 0;`；把正文放进方法自己的签名后 javac MUST 接受，`int cannot be converted to boolean` 不得出现

#### Scenario: A boolean call result as a condition

- **WHEN** 分支测试的操作数是返回 descriptor 为 `Z` 的调用结果（源码形状 `if (flag()) { return 1; } return 0;`）
- **THEN** 呈现 MUST 是真值测试 `if (flag())`，MUST NOT 是 `if (flag() != 0)`；javac MUST 接受该正文，`incomparable types: boolean and int` 不得出现，且执行结果与原 class 相同

#### Scenario: An unproven boolean context is refused

- **WHEN** 上下文要求 boolean，而该值的 boolean 类型没有证据（不是返回 `Z` 的调用结果、不是 descriptor 声明为 `Z` 的参数或由其声明的局部、也不是 boolean 字面量/否定）
- **THEN** 该区域 MUST 拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI 与物理方法），MUST NOT 以 `1`/`0` 或与 `0` 比较的形式发布；拒绝产物按既有 refusal 契约可定位

#### Scenario: An int context keeps its int shape

- **WHEN** 方法真正返回 `int`（descriptor 返回 `I`），或分支测试比较两个 int 值（`if (x != 0) { return 1; } return 0;`）
- **THEN** 呈现 MUST 保持 `return 1;`/`return 0;` 与 `if (arg0 != 0)` 的整数形态，MUST NOT 改成 `true`/`false` 或真值测试；本要求 MUST NOT 用普遍改写成 boolean 换取反例通过

#### Scenario: An integer comparison keeps its integer literals

- **WHEN** 分支测试是整数二元比较，且其中一个操作数是 `0`/`1` 字面量（源码形状 `if (1 == n)`、`if (0 < n)`、`if (1 < n)`，字节形态为 `iconst_1; iload_0; if_icmpne` 一类）
- **THEN** 呈现 MUST 是 `if (1 == arg0)`、`if (0 < arg0)`、`if (1 < arg0)` 的整数文本，MUST NOT 是 `if (true == arg0)`、`if (false < arg0)`、`if (true < arg0)`；把正文放进成员自己的签名后 javac MUST 接受

#### Scenario: Either side of the comparison keeps its own spelling

- **WHEN** 常量 `0`/`1` 出现在整数二元比较的右侧（源码形状 `if (n == 1)`、`if (n > 0)`），或该比较是相等与大小两个方向中的另一个
- **THEN** 两个操作数 MUST 各自保持整数文本（`if (arg0 == 1)`、`if (arg0 > 0)`），MUST NOT 出现 `if (arg0 == true)`、`if (arg0 > false)`；本要求 MUST NOT 只检查左操作数，MUST NOT 只以一个比较方向作为证据

#### Scenario: The same literal is spelled by its context

- **WHEN** 同一段字节里的一个 `iconst_1` 分别被二元比较、被证明 boolean 的操作数的零值测试、以及 `Z` 方法的 `return` 读取
- **THEN** 它在二元比较里 MUST 是 `1`，在真值测试与 `Z` 返回里 MUST 是 `true`（`if (arg0)` 与 `return true;`）；文本由上下文决定，MUST NOT 由字面量自身的形状或渲染顺序决定

#### Scenario: A proven boolean operand keeps the truth test

- **WHEN** 零值测试的操作数是一次返回 descriptor 为 `Z` 的调用，或一个由 descriptor 证明 boolean 的参数/局部（字节形态 `invokestatic flag()Z; ifeq` 一类）
- **THEN** 呈现 MUST 仍是非比较的真值测试（`if (flag())`、`if (!flag())`、`if (arg0)`），MUST NOT 因为本修正退回 `if (flag() != 0)`；本条与整数比较的修复 MUST 同时成立
#### Scenario: An append parameter keeps its own width

- **WHEN** 一个 `StringBuilder.append` 调用的参数被窄化（例如 `append((int) c)`，`c` 为 `char`）
- **THEN** 呈现文本 MUST 让该片段按 `int` 求值（例如 `(int) arg0` 形态）；当 `c == 'A'` 时执行结果 MUST 与原类一致（`"65!"`），MUST NOT 得到 `"A!"`（C01）

#### Scenario: A call argument meets the parameter's type

- **WHEN** 实参的呈现类型与目标形参类型不一致（`char`/`short`/`byte` 与 `int`/`long` 混排）
- **THEN** 文本 MUST 显式转换到形参要求的类型，或按证据不足拒绝该区域；MUST NOT 靠上下文隐式转换发布（C02）

#### Scenario: Classification planes do not stand in for values

- **WHEN** 一个样本的四个报告平面全部为 Complete 且文本能被 `javac` 接受，但两者执行结果不同
- **THEN** 该样本 MUST 被当作失败；验收证据 MUST 是原类与呈现文本在同一输入集合上的执行对照，MUST NOT 是平面状态或编译通过（C01/C03）
