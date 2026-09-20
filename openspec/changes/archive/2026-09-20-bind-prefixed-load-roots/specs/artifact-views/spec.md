## ADDED Requirements

### Requirement: CLI exposes physical tree discovery without choosing roots

JSON CLI SHALL 提供显式 artifact-tree 枚举操作，返回与库同一物理范围的 container、entry、layout evidence、coverage、execution 和 diagnostics；调用方能够使用真实返回身份声明加载位置。普通枚举 MUST 保持不自动递归；tree 枚举也 MUST NOT 自动生成 loader 委派、root 顺序或选中类的结论。

#### Scenario: Enumerate then recover a prefixed class

- **WHEN** 调用方经 CLI 显式枚举一个 WAR，再用返回 snapshot/entry/container 身份与自行声明的 prefix 发起方法请求
- **THEN** CLI 的枚举和恢复结果与相同输入、环境及预算的库结果语义一致，不需手工伪造 child identity 或改写 raw name

#### Scenario: Partial tree in the CLI

- **WHEN** 枚举遇到坏 nested archive、预算不足或取消
- **THEN** CLI 保留库报告的可靠前缀、诊断与未完成范围，不返回完整空树，不自动重跑或扩大限制
