## Context

见 proposal。已接受的原形基线在 `em23-variable-postfix-loop/baseline-root-v1`：原2/JADX4成功，旧Jarde4完整源码缺return。新普通控制基线 `one-arm-loop-controls/baseline-root-v1` 由root实际执行31命令、111文件；原2/JADX4成功，乘法CLI的Jarde4完整源码缺return且零runtime。三个带前缀方法都解释为 branch0 `ArmsDoNotMeet`；`noPrefix` 已structured，但回跳goto@14未出现在source map，另行拆分。新基线独立验收待执行，collector完成不等于产品成功。

root复核 `Frame::arm`、`region_at`/loop-header分段、one-arm caller、`continue_early_return_arm`、所有相邻loop continuation及`header_tested_loop`。现有probe证实原形外层ipdom=45，并未记录递归arm返回。源码候选链是setup@6形成Straight前缀，在header13返回next13；one-arm caller只尝试boolean early-return shape而拒绝。带method身份的临时patch已准备但未编译，实施前须动态确认。Atlas scoped context也确认该helper的唯一caller在one-arm分支，不把它误当通用arm driver。

## Goals / Non-Goals

**Goals:** 完成一个准确单臂中Straight prefix + 已证普通Loop + 可选Straight tail的局部组合；普通while四方法和JADX原形完整类原样重编、运行、物理身份及来源验收。非空前缀不按固定BCI或固定一块长度匹配。

**Non-Goals:** 任意region续接框架、多个循环串接、未证明的多入口/多出口与新finally/switch交互、SSA或类型规则放宽、foreach/++呈现扩展，以及latch来源修复。`noPrefix` goto14来源另有独立change；最终来源接受依赖它关闭，不能在新candidate中把missing14豁免为通过。

## Decisions

1. **从one-arm caller补局部续接。** 在原arm返回非外层join且下一块为未认领fresh loop header时，沿用同一arm Frame再次调用统一`region_at`，复用现有自然循环、scope/handler、递归与预算门禁。前缀必须为已证直线run，其首块为所选arm入口、最后正常边准确到header；不能把外层LoopTarget的header/continue回边作为新entry。先动态证实此停止点；若证据否定，不直接实施假定路径。
2. **完成边界与ownership后拼接。** 接受一个完整structured循环run，实际Region出口与返回next相同，完整循环集合与物理owner一致；所有外部入口仅为prefix末块到header，正常出口仅到一个目标。该目标是外层join时结束；否则只继续一个独占Straight尾部，所有正常前驱/后继及最终next准确到join。prefix/loop/tail互不重叠、新增visited恰等于候选Region持有块，最终仍经现有全树overlap/coverage检查。所有边和owner扫描使用已有charge/poll，不靠BCI地址顺序推定流程。
3. **复用Run与Region::Sequence，不增机制。** `continue_inner_join_arm`消费的是已完整内层If的join尾部；`continue_effectful_loop_arm`与`continue_multi_return_loop_arm`依赖各自固定拓扑/特殊finally证书，均不是本例的证书。借鉴其局部续走模式，不全局启用其前提。必要的短私有helper只能服务此组合，不新增Frame字段、公共IR、pass或transaction。
4. **失败沿既有整方法拒绝。** 不闭合时设置既有unclosed-tail并quoted-whole，不留下可执行半臂。Stop原样传播、预算消耗不回滚；不增加通用回滚状态。只有实施选择继续尝试其他结构时才需要已有visited snapshot复原，而本方案避免此alternative。
5. **借鉴JADX遍历职责，保留本项目证据强度。** 已核JADX `RegionMaker.makeRegion`的有界traverse-loop、`traverse`对loop-start优先交LoopRegionMaker和后续next；它继续收集同一region直至stack出口，不因前缀已存在而终止。其processed block重入可标DUPLICATED并继续，本项目不照搬该容忍路径；仍要求唯一ownership。依赖与算法均已有，新增库无法补齐caller停止契约，故无外部库/许可变化。
6. **对照边界按模块分开。** 解码/Java8验证及双JDK选择复用既有reader/runtime；修复只作用source recovery。完整源不删成员、不借原class/helper、不手改正文，仅Runner必要package适配。default/all比较正文与来源，不强求可选evidence记录或usage相等。完整来源候选不能掩盖noPrefix旧缺口：先记录，依赖独立修复后以fresh CLI再核全部物理BCI。

## Risks / Trade-offs

- [prefix外有另一条入口或tail属于另一臂] → 正常/异常入边、scope和exact owner核验；第二入口/不同出口/已认领/外层continue反例保持拒绝。
- [循环后tail被移到条件外或多执行一次] → 原形与四方法全源码双JDK对照，0/1/4轮及两种极性核raw；不只检查有while文本。
- [循环已有fallback却被wrapper当成完整] → 只接受完整循环region与闭合next；其余整方法quote，不改fallback内容与生产者来源。
- [Stop被当作失败继续走] → 预算/取消/递归中断测试如实核execution，原传播路径不改；临时diagnostic撤销后冻结产品CLI。
- [新控制基线的旧goto14缺口让来源门禁失真] → 独立来源change关闭后再做最终all-BCI接受；本片task不能单凭runtime或quality勾完整来源验收。

无需迁移或兼容层。Rust仅在5GiB机器/1GiB本仓target守卫满足后串行执行（用户已明确授权下调机器下限，保留实时中止及完成清理），临时probe撤销前不提交生产源码。
