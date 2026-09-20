## ADDED Requirements

### Requirement: Task-oriented CLI is a thin adapter over library operations

CLI SHALL 提供任务链命令，每条命令直接调用相应的库操作，并把该操作的报告（含 coverage、execution、diagnostics 与 usage）作为唯一结果来源；CLI MUST NOT 重新扫描 artifact、重新选择目标、重新分析文本或补造结果。同一请求 SHALL 支持文本与 JSON 两种输出，两者 MUST 由同一份报告派生且语义一致。诊断 MUST 与正文分离：正文进入标准输出或输出文件，诊断进入独立字段或标准错误，不得混入代码正文。输出文件选项 SHALL 在写入前执行与标准输出相同的 `output_bytes` 检查，文件内容 MUST 与标准输出模式一致，执行未完整时 MUST NOT 写出伪装成功的文档。命令 SHALL 使用显式、稳定、可区分的退出状态：成功为 0，用法/输入错误、需要调用方选择（名称歧义）与执行未完整（Partial/Cancelled/Stopped）各自给出不同的非零状态。任务链内的后续命令 SHALL 接受前一条命令返回的物理身份，MUST NOT 要求调用方手工重拼身份。

#### Scenario: Text and JSON come from one report

- **WHEN** 同一请求分别以文本与 JSON 运行
- **THEN** 两者的方法身份、content/quality、coverage、execution 与诊断集合一致，仅渲染不同，且没有额外的 artifact 读取（验收 A13、A16）

#### Scenario: Diagnostics are separated from the body

- **WHEN** 恢复产生诊断并以文本模式输出
- **THEN** 诊断不进入代码正文；JSON 输出把 diagnostics 放在独立字段，正文与诊断都可定位（验收 A13）

#### Scenario: Output file matches stdout

- **WHEN** 同一请求使用输出文件选项
- **THEN** 文件内容与标准输出模式一致；预算不足或取消时保留真实停止状态，不写出部分成功文档（A14）

#### Scenario: Exit statuses are explicit

- **WHEN** 请求成功、用法/输入错误、名称歧义需要选择、或执行未完整
- **THEN** 退出状态分别为 0 与三个相互不同的非零值，原因可定位；执行未完整即使有可靠前缀也 MUST NOT 以 0 退出（验收 A14）

#### Scenario: Task chain reuses returned identities

- **WHEN** 调用方先列类、列方法取得物理身份，再用该身份发起恢复
- **THEN** 后续命令按该身份执行，读取范围与库操作一致，不要求手工重拼或重新查找（验收 A16）
