# Design

## Context

动机和文档范围见proposal。当前产品aadf71的4905源已绑定最终CLI，修复自身CI独立运行；本片不得改动该产品源码或把其CI称作文档新验收。原snapshot-hierarchy的MVP文字与后platform、heterogeneous-array已交付文字缺少历史范围衔接；归档的class-source停止设计已明确Mid与terminal的区别。

实际OpenSpec指令对无产品/需求行为改变的纯文档改动要求skip_specs，故采用该配置，specs状态应为skipped，main的17条java8需求不在本片增加。本片保留design，是因为跨历史规划的范围解释需要先固定技术边界。

## Goals / Non-Goals

**Goals:** 注记只解释旧MVP与各后续批准范围；固定必要source/intermediate读取和header名称terminal的差别，使负控引用可以复核。

**Non-Goals:** 无Rust/CLI、fixture、预算、选择或恢复变更；无新需求、终点查找、完整assignability服务或产品测试矩阵。method站点快照外用户interface终点、terminal重复物理定义的独立政策保持未决，不能在文字中默认允许或拒绝。

## Decisions

1. 只在实际混淆点追加有标题的历史范围注记和相对链接，不改旧需求的WHEN/THEN、旧raw或验收结果。保留two-sided为原支持子情形，并链接后续已批准的具体站点；不把platform标题当成用户父类从未获准的证据。
2. 说明source/current IR与实际walk需要的intermediate按照既有环境选择和读取规则取得证据；缺失/歧义/执行停止沿各自原契约。可信header命名terminal不表示已经选中、读取或验证terminal的物理定义，也不能据此补齐未读后续边。所有描述必须能回指既有正文/源码，不另立普遍关系政策。
3. 引用已归档preserve-class-source-dependency-proof-stops/design.md现有Mid负控说明，不重写档案、也不把其规划叫作当前未归档工作。H1/Greet terminal删除/重复不作为必读依赖负控；当前消费者的Report/Error停点契约不在这里重定。
4. 以skip_specs完成纯文档交付，无delta同步。曾有cache-only ADDED草稿但未采用：实际CLI要求不为满足校验发明需求，且本片无产品行为改变。该历史草稿不等于项目已立项/已执行。本片无需外部库、依赖或自定义框架。

## Risks / Trade-offs

- [注记被读作新语义] → 明示历史时间/适用站点与未决项，peer逐句核对引用。
- [名称终点被写成物理目标] → 禁止该额外结论，检查source/intermediate/terminal三种义务表述。
- [范围漂移到产品修复] → diff仅允许必要OpenSpec文档注记，原Rust、fixture、旧raw/结论和main需求全文保持原样。

## Migration Plan

固定相关文档版本与引用，做最小注记和独审，实际运行项目两版strict并保存命令/版本/raw/pins，最后按skip_specs正常归档。无数据迁移，若文字误述则撤回新增注记；不回滚或重跑已验收产品。旧source/full-runtime成果只作历史引用，不加新通过数。
