## Context

动机和分层边界见 proposal。真实完整 class SHA-256 `74d632baa95485b7233476984a56598b9689cfa1552f965b5a1dd87704ad1525` 在双 JDK 通过验证；lib-v9 原始诊断证明第一个失败为内部克隆 `(14,[0])` 被每块直接 Call/Return 校验误拒，Frame/SSA 尚未执行。Atlas 局部调用事实确认生产发布入口为 canonical build。既有 drain 已证明普通/异常边保持路径、Call push、Return pop；assemble 已建立 outgoing 并独立计算方法 root 的 unreachable。

## Goals / Non-Goals

**Goals:** 修正已有规范化产物的覆盖证明，复用现有装配邻接和预算。不新增公开产物或通用图机制。

**Non-Goals:** 不重做 returnAddress、CallContexts、handler 归属、融合、Frame/SSA 或完整 verifier；不改变普通非 legacy 图预算，不把覆盖可证误作方法入口可达，不扩张 Java 源码恢复。

## Decisions

1. **覆盖是 rooted 闭包而非直接调用入边。** 在 assemble 将既有邻接列表元素改为借用 canonical edge，继续用它做原 root reachability；额外覆盖从全部现存 `path=[]` 节点开始。只按真实边前向传播，Normal/Exception 两端同路径，Call 目的路径等于源路径追加其 call_site，Return 源路径等于目的路径追加其 call_site。不匹配的路径转移拒绝产物。所有 clone 必须进入闭包；断开环、未覆盖 clone 返回都不能自证覆盖。旧 postcondition 的逐块直接 Call/Return 检查移除，其来源/handler/边完整性检查仍保留，克隆证明属于有 Budget 的 assembly 发布前校验。

2. **死上下文与覆盖独立。** 全部 path=[] 是规范化已经保留的 method-frame 节点，包含明确死 callsite。它们仅为覆盖种子；原 root reachability 算出的 unreachable 不变。不能将全部 seeds 改成 live，也不能丢弃死 clone。嵌套 Return 只有已覆盖 source 能支持 outer continuation。

3. **复用既有标准集合及收费。** 邻接已存在，不另造第二套 owning CFG/索引；覆盖 seen/worklist 借用现存 IDs，避免复制路径 Vec。新增 retained/queue 槽在增长前按 IrItems 计费，实际 pop、edge 检查及路径比较按 AnalysisSteps 计费并通过 charge poll。仅在存在克隆的 legacy 图执行新增工作，普通图费用保持不变。停止传播 Norm::Halted，不转为 Norm::Unproven，published item 仍在全部校验之后。Assembly 现有取消 checkpoint 保留，不新增 phase 或生产 hook。

4. **没有需要引入的第三方库。** 本项目 std BTreeMap/BTreeSet/worklist 和既有 Budget 足以完成有限闭包；petgraph 已为 raw CFG 使用，但迁移到额外图表示会增加存储及收费，不能改善这里的路径身份校验。无新增依赖、维护或许可证负担。

5. **先 root 实测再验收。** 新类正例必须经真实 Reader 使 CanonicalCFG/Frame/SSA 可取得并固定两套克隆实际身份；原 class/JVM 验证与 Jarde 分析区别记录。图级 mutation 只验证发布前覆盖层，不能冒充 JVM 合法输入。已有历史 shared/nested/handler/fusion、死调用点和预算取消全部回归；复合条件 clone 负例必须实际到达 region guard 才计该父项完成。

## Risks / Trade-offs

- [仅检查 frame 存在 Call 会错误接受断开 clone] → 检查每个 clone 的 rooted 前向闭包，加入同路径断开、自循环和未覆盖嵌套返回负例。
- [coverage seeds 把 dead 改成 live] → 保留 root reachability 及 unreachable 的独立真值，永久测试准确集合。
- [新增扫描/临时槽漏收费或吞 Stop] → 增长前收费，节点/边/路径工作逐项预算，直接传播停止并测试无产物。
- [错误混入父片] → 独立 change、任务和证据；父片只消费规范化结果。

## Migration Plan

无公开接口或持久 schema 迁移。root 审阅 Luna 私有补丁后串行应用，5GiB 最低空闲/target1GiB、一秒进程组守卫运行测试并清理；失败保存，未通过不勾项。自身产品 CI、提交推送和干净交付仍作为父片最终交付门，不借历史 CLI 或 CI。
