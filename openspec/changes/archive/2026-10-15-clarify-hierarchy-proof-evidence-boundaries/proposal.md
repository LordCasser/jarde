# Proposal

## Why

旧snapshot-hierarchy规划将“两端均为快照内物理定义”作为MVP边界，后续platform与array条款已经容许特定的header名称终点；历史文字缺少交叉说明，容易被误读成当前通用拒绝规则。仅补文档范围和引用，避免将已读source/intermediate与未读terminal物理身份混同。

## What Changes

- 在旧snapshot、后续platform和array规划的必要位置添加最小历史范围注记与相互引用，保留原正负场景及历史验收原文。
- 说明实际需要读取的source/intermediate必须沿已有运行环境取得证据；可信header直接命名terminal时，按各自已经批准的站点范围解释，不额外声称选中物理target。
- 引用已归档preserve-class-source-dependency-proof-stops设计中选择Mid负控的现有依据，保留用户父类追认及two-sided已支持子情形。
- 明确method站点快照外用户interface终点、terminal重复定义政策不在本片裁定范围；不把某一站点的验收扩成全部Java可赋值性。

## Capabilities

### New Capabilities

无。本片不新增产品能力或需求。

### Modified Capabilities

无。本片采用实际schema支持的skip_specs，只澄清现有规划/设计文字的历史和引用，不修改main需求或新增ADDED条款。

## Impact

仅影响OpenSpec文档注记与交叉引用。无Rust、CLI、公开schema、依赖、fixture、生成Java、预算计费或停止行为变化；旧raw与验收结果保持原字节。本片不重跑产品、完整源码或双JVM测试，也不借其他产品提交CI完成文档验收。

前提是引用实际存在且可以固定版本复核；交付只需文档对照、小范围独审、两版OpenSpec strict及按skip_specs正常归档，无delta需要同步。主产品aadf71的自身CI和正常交付仍是独立工作，本规划不改变其验收。
