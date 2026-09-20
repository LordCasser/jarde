## ADDED Requirements

### Requirement: CLI navigation reuses library listing and tree discovery

CLI 导航命令 SHALL 仅使用库的类/成员列举与显式 artifact-tree 枚举作为发现面，MUST NOT 增加第二套目录扫描、路径推断、解包或自动 root/loader 构造。CLI SHALL 原样保留库报告中的 container/entry 身份、重复物理定义、coverage、execution 与诊断，并与相同输入、环境与预算的库结果语义一致。列出条目 MUST NOT 被解释为 MR 选择、loader 策略或可加载性结论。

#### Scenario: Tree enumeration is the discovery surface

- **WHEN** 调用方要处理 WAR 中 `WEB-INF/classes/` 下的类
- **THEN** CLI 先用显式 tree 枚举取得真实 container/entry 身份，再由调用方声明 prefix root；不按路径猜测，也不使用第二个扫描器（验收 A07、A08）

#### Scenario: CLI listing preserves library evidence

- **WHEN** CLI 列举类或成员
- **THEN** 结果与同输入、同预算的库列举语义一致，保留 origin/ordinal、重复项以及 Partial/Cancelled 状态（验收 A07、A14）

#### Scenario: Broken child is not repaired by the CLI

- **WHEN** 某个 entry 无法建立为 child container
- **THEN** CLI 保留诊断与可靠前缀，不自行解包、重扫或扩大预算（验收 A08、A14）
