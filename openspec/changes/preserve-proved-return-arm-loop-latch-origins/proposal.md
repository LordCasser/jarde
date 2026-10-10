## Why

CF07 倒序查找能生成并运行正确源码，但唯一物理回边 goto@25→5 没有源码映射。root 的同一次真实 IR 诊断已经证明末尾 If 的返回分支与独占回边分支；现有来源证明只读取循环体最后直接 Straight，应沿现有证书覆盖这一已确认形状。

## What Changes

- 复用自然环、SSA、完整 canonical CFG、Region 所有权与 gateway_origins，保留末尾无 join If 的准确回边来源。
- 对返回分支、唯一回边、准确循环头与完整出边作有限证明，保留所有正文及既有来源。
- 添加真实形状反例和预算/取消回归，使用新 CLI 独立验收完整 CF07 与旧循环控制。
- 生产应用前置为 If 来源产品 1f386686c6a31813254a85fed48d2af0af84a61c 的自身 CI 接受及 clean 交付。本片不扩 ForHeader 初值、不做递归回边搜索或异常 Frame 修复。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明循环末尾返回/回边 If 保留完整循环语句的物理回边来源。

## Impact

限 jarde-java 既有循环尾来源证明及永久回归、完整类证据。无新 IR 字段、Frame、pass、依赖或 CLI。71 单元/612 文件分母不变；CF07 的 computed-init for 呈现和其他未验形态单独记录，不能因本片全物理 BCI 覆盖而宣称整单元追平。
