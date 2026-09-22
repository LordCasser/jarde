## MODIFIED Requirements

### Requirement: Bounded archive reading

系统 SHALL 有界读取 STORED/DEFLATED entry 并校验 CRC/size；加密、不支持的压缩、非法边界、超限和取消 MUST 显式返回错误或部分状态。系统 MUST 不向用户路径解包或执行 artifact。快照 MAY 暴露一个**受信读取的复用入口**：调用方按定义身份请求一次已完成的读取，命中即返回该读取的字节与该读取建立的 digest，且**本次读取零字节、零费用**；不命中、容量不足或复用时未启用时 MUST 走既有有界读取路径，其校验与限额语义不变（见 `facts-cache` 的定义读取复用要求）。

#### Scenario: Decompression limit

- **WHEN** entry 声明或实际展开字节超过预算
- **THEN** 读取终止并返回命名的预算维度，不返回空内容冒充成功

#### Scenario: Nested archive locator

- **WHEN** WAR 含 WEB-INF/lib/library.jar
- **THEN** P0 枚举保留该 entry，并声明嵌套内容尚未递归扫描

#### Scenario: A reuse hit reads nothing

- **WHEN** 调用方以与保留时逐项相同的定义身份请求一次已完成读取，且复用已启用、容量允许
- **THEN** 返回该读取的字节与其 digest，本次不读取容器、不计 `archive_entries`/`read_bytes`/`entry_bytes`；调用方仍须核对它声明的身份与返回的 digest/length 一致

#### Scenario: A reuse miss takes the ordinary path

- **WHEN** 身份不支持、复用未启用或容量被拒绝
- **THEN** 走既有有界读取路径：CRC/size 校验、预算维度与取消语义与从前完全相同，MUST NOT 因该入口的存在而放松任何校验
