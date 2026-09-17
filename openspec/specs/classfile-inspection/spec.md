# classfile-inspection Specification

## Purpose
提供可独立使用的 classfile Header 和字节码检查能力，为后续查询与反编译共享原始结构、方法身份和字节位置，同时明确区分结构读取、合法性检查和源码恢复。

## Requirements

### Requirement: Lossless bounded headers
系统 SHALL 返回类/成员完整名称、descriptor、flags、继承信息和 attribute spans，保留 MUTF-8 原始字节及 UTF-16 code units。Header 请求 MUST 不解码方法指令或建立 IR。

#### Scenario: Unknown attribute and malformed method body
- **WHEN** 类含长度合法的未知 attribute，且某方法 Code 指令非法
- **THEN** Header 可以读取外壳并保留未知 attribute 位置，不能据此声称方法合法

#### Scenario: Isolated surrogate
- **WHEN** 常量池字符串含 NUL 或孤立 surrogate
- **THEN** 返回精确的原始字节、UTF-16 和转义显示，不使用 lossy 值作为身份

### Requirement: Shared bounded instruction inspection
系统 SHALL 按请求方法解码指令，返回 BCI、opcode、width、原始字节 span、适用的 CP index 及异常表顺序；switch/wide MUST 按 Code 起点确定边界。未知/截断指令 MUST 停止当前方法并返回部分状态。

#### Scenario: Wide and switch instructions
- **WHEN** 方法包含 wide、tableswitch 或 lookupswitch
- **THEN** 指令边界与真实 bytecode 偏移一致，不把 operand 当作新 opcode

#### Scenario: Unknown opcode
- **WHEN** 方法中出现未知 opcode
- **THEN** 返回可靠前缀、错误位置及 Partial，且不继续猜测后续边界

### Requirement: Capability based versions
系统 SHALL 分别报告结构读取、已执行 dialect 检查与未执行 verification；45–52 的版本/minor 检查 SHALL 有逐项测试，53–71/preview/未来 major SHALL 明确为结构探测或未支持。

#### Scenario: Future or preview dialect
- **WHEN** 输入 major 超出基线或使用未实现的 preview dialect
- **THEN** 不宣称完整支持；Strict 和 Forensic 的拒绝/探测行为可区分

#### Scenario: Historical minor version
- **WHEN** 输入为 45.3、45.65535、51.1 或 52.1
- **THEN** 结构版本规则不能一律按 minor=0 拒绝，45.65535 不视为 preview；Java 8 版本范围检查单独标记 52.1 超出 52.0 上界

#### Scenario: Invalid modern minor
- **WHEN** major≥56 的输入使用非 0/65535 的 minor
- **THEN** Strict 拒绝版本违规；Forensic 仅在边界可读时提供带版本诊断的结构探测结果
