## ADDED Requirements

### Requirement: Proved enum constants recover literal String varargs

对于完整的 Java 8 普通 enum group，class-source 只有在同次选中 run 证明唯一物理构造器 `(Ljava/lang/String;I[Ljava/lang/String;)V`、`ACC_VARARGS`、源 Signature `([Ljava/lang/String;)V`、隐式 `Enum(String,int)` 调用，以及第三参数只写入唯一实例 `String[]` 字段时，才 SHALL 恢复 `String...` 构造器和有序常量实参。证明 SHALL 将每个常量绑定到其物理 enum 字段、ordinal、数组分配、构造调用、字段写入，以及已有的完整 `$VALUES`、`values()`、`valueOf()`、构造器调用 census 和初始化 suffix census。生成源码 SHALL 保留每个数组的有序字符串值和新数组构造语义。

本切片只接受最多 64 个元素且每个 classfile modified UTF-8 字符串字节均为 ASCII 的 literal。非 ASCII 字节、modified UTF-8 NUL 编码和无效 UTF-8 SHALL 整组拒绝；不得使用 lossy 解码。字符串使用既有 Java literal 转义函数输出。

#### Scenario: Literal arrays and empty varargs
- **WHEN** 完整普通 enum group 的常量实参是新分配的 `String[]`，长度不超过 64，元素在 `0..n-1` 按序写入 ASCII 字符串 literal，且包含实际分配的零长度数组
- **THEN** class-source SHALL 输出合法 `String...` 构造器和对应有序 literal 实参；Java 8 重编译和运行 SHALL 保留原数组长度、值、构造顺序和不同常量的数组身份

#### Scenario: Unsupported array shape refuses the whole group
- **WHEN** 任一常量数组被别名、读取、逸出、重复或乱序写入、由非 literal 填充、传给额外 consumer、替换为其他数组类型或 null，或任一字符串超过 ASCII 子集
- **THEN** 整个 enum 投影 MUST 原子拒绝，保留普通物理字段、初始化器和构造器 fallback、markers 与证据；MAY NOT 选择性投影个别常量

### Requirement: String varargs proof consumes exact same-run Code and source metadata

数组证明 MUST 消耗完整、连续且无 handler 的同次 raw Code/BCI 初始化前缀：每个常量包含有界非负长度 literal、一个精确的 `anewarray java/lang/String`，随后每个元素恰好包含一组 `dup`、精确 index literal、`ldc` 字符串 literal 和 `aastore`，再紧接唯一预期的构造调用和常量 `putstatic`。证明 MUST 检查 opcode operand、constant-pool reference、指令 width 与 BCI、栈使用形式、完整类和 member 表、唯一构造器和字段目标，以及所有既有 group gates；AST 文本和 JADX 输出 MUST NOT 替代缺失证据。零长度数组也 MUST 证明真实分配。物理构造器的源 Signature、descriptor 与 varargs flag MUST 一致，且唯一用户可见效果为将该数组存入自身唯一实例字段。

#### Scenario: Incomplete metadata or interrupted proof
- **WHEN** Code、Signature、flag、字段/构造器 identity、symbolic reference、指令覆盖或 group use census 缺失/歧义，或预算/取消停止依赖读取
- **THEN** SHALL NOT 发布 String-varargs enum 投影；停止状态 MUST 保持 stopped，物理成员仍可独立查询

#### Scenario: Extra initializer work remains unsupported
- **WHEN** 前缀包含未证明指令或副作用，或 suffix 不符合已有 terminal-return 或完整 static-assignment 规则
- **THEN** 整组 MUST 拒绝，而不是丢弃或移动额外行为

## Non-Goals

本变更不恢复任意 Java 数组表达式、非 literal varargs、非 ASCII literal、超过 64 个元素、重载 enum 构造器、匿名常量体、其他 varargs 元素类型或一般构造表达式。它不改变公开 report 格式或单方法恢复。
