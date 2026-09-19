# modern-classfile-semantics Specification

## Purpose
为现代 classfile 和运行时版本提供可验证的结构与语义状态，使模块、nestmate、condy、record、sealed、现代拼接和 preview 规则不会被错误降级或误报为完整源码恢复。

## Requirements

### Requirement: Version and feature registry

系统 SHALL 以 release 绑定的 Registry 记录 major/minor、preview、constant-pool tag、attribute 合法位置/版本、flags 和 opcode 约束，并将 parse、dialect validation、verification、output-level 状态分开。

#### Scenario: Preview and future release

- **WHEN** 输入是未登记的 preview 组合或高于当前 registry 的 major
- **THEN** 系统保留可读结构（若边界可解析），并明确标为 unsupported/partial；不得把显示成功当作 dialect 或 verifier 成功

#### Scenario: Version-specific flags

- **WHEN** record、sealed、module 或 nestmate attribute 出现在不适用版本/位置
- **THEN** 返回对应版本/位置诊断，不用类名、方法名或路径猜测其语义

### Requirement: Modern structural semantics

系统 SHALL 结构化表示 module/package/uses/provides、nest host/members、constant dynamic、record components、permitted subclasses 和现代 string concat，并保留原始 CP、attribute、bootstrap、BCI 和 origin。

#### Scenario: Nested condy graph

- **WHEN** dynamic constant 通过参数继续引用 dynamic constant 或 bootstrap 子图
- **THEN** 系统使用 visited/深度/边数预算返回共享依赖图和 use-site 路径，不执行 bootstrap 或无限递归（验收 A05）

#### Scenario: Record and sealed declarations

- **WHEN** 版本规则确认 record components 或 permitted subclasses 合法存在
- **THEN** 输出结构事实与版本证据；Java 8 output level 报告冲突或 fallback，不伪造等价降级
