## Why

本 change 是 [性能优化专项](../optimize-demand-workloads/proposal.md) 的 O1 实施子项，负责消除单 container 访问中的越界工作和重复工作。基线中 S2-001 一次方法请求计费 5,900 entries / 7.0 MB，而一次树枚举为 2,799 entries / 3.67 MB；这证明了重复枚举/展开，但尚无阶段耗时占比，不能据此承诺整体倍数或亚毫秒延迟。

## What Changes

- 将已声明 container 的查名与读取收敛为沿 origin chain 的定向访问，不访问未搜索的 sibling container。
- 贯通单次操作的定位与读取；即使跨请求 cache 关闭，仍消费本次已取得的权威目录与 backing，不因阶段切换重做相同工作。
- 在 reader 内复用经过验证的 container backing、完整目录及 raw-name 到物理 entries 的定位表；跨请求复用显式开启，按内存权重有界。若现有 cache 构造 API 需改为 entry/byte 双上限，按 **BREAKING** 迁移调用点，不保留平行旧接口。
- 保留完整扫描、重复项、origin、预算及取消语义；缓存命中报告复用事实，不能伪造本次 I/O。
- 按总专项 W1/W2/W5 测量冷单次、同 snapshot 多方法和容量退化；分别报告准备成本、请求延迟和总成本，不把不同分母混用。
- 分阶段比较原始路径、定向直接路径、显式复用的 cold/warm/容量不足路径，隔离既有 CP/Header cache 的收益；记录阶段耗时占比、工作计数、内存代价和结果 fingerprint。
- 按总专项的阿姆达尔与端到端证据门禁报告容器子项的收益、代价和启用/停止结论；其它方案由总专项独立调查，不进入本子项实现任务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `demand-resolver`：已声明 container 的定位只访问其必要祖先链与被搜索位置；同次操作贯通定位与读取。
- `facts-cache`：增加按真实需求构造的 container facts、身份、完整性、资源上限与复用边界。
- `measured-execution`：增加容器重复请求的可归因测量及完整/语义确定性对照；通用专项决策门禁由总 change 声明。

## Impact

前提为 P1 物理身份/nested replay、P2 显式 roots、P5 测量及 cache 边界已交付。影响 reader 的 artifact/cache/budget、jvm providers 和性能测试；不新增 crate，不升级依赖。现有恢复正确性修复的回归必须在候选提交继续通过，历史 `cd6f2f0` benchmark 不能作为新提交的质量基线。

不包含 prefix root、默认打开任何 cache、查询 class 共享实现、方法定位表、批量 API、分页重设计、CP/Header ownership 重构、resolution/IR/source cache、预加载/预取、并行、持久索引、single-flight、mmap、JSON 微优化或新的 session 框架。这些方向只能在本主线完成并重测后另行决策；性能专项不承担其它正确性 change。关联分析见 [benchmark review](../../benchmark-review.md)。
