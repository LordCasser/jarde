# §2 O1–O8 分项调查的证据锚点（由主 Agent 登记）

每条的完整结论在其证据文件里，形状固定为八段：问题 / 测量形状 / 必要工作证据 / 理想上界 / 资源与语义代价 / 负向实验 / 处置建议 / 未确认项。此处只登记**结论行与指针**，供 §3 准入引用。

| 项 | 证据 | 必要工作证据（摘要） | 理想上界 | 处置建议 |
| --- | --- | --- | --- | --- |
| O1 | `evidence/o1-container-lookup.md` | W1 冷请求目录解析 21/11、物化 0（bcprov）；第二次请求 +2/+2 命中且不再解析；往返目录解析 11 对 31（roomy/tiny/none） | 定向 2+2（728 B）对整树 26+25（7866 B） | D/C/W 维持已交付；F 维持"满则拒绝"；历史整树路径 **不可归因**（符号已不存在） |
| O2 | `evidence/o2-class-reads.md` | W1 每请求仍 1 物化 / 0 准备 / 1 解码；W2 9 次物化；W3 两臂 36 读+36 准备 对 8 读+4 准备；`class_source` 1/1（双读取已关闭） | 同进程直接对照 request 2206 → 600 µs（3.7×），类读 4.5×、准备 9× | **准入候选**："操作内复用同一次已选读取"的最小实验；不建议全局 `MethodIr` |
| O3 | `evidence/o3-query-demand.md` | 首屏 18/42 µs；小页 548 页/538.5 ms 对大页 18 页/260.5 ms；**条目级重扫 = 0**（Σscanned == items） | 530 个页边界 = 278 ms（W6a 1-worker request 的 7.5%） | item 级停止：先定 cursor/coverage/diagnostic 子 spec 再实验；boundary 重扫**不存在**，不立项 |
| O4 | `evidence/o4-class-batch.md` | W3 两臂同结果；W6a `classes_prepared` 2430 对 15003 方法（去重后 6.2 方法/类）；1/2/4/6 worker request 3705 → 2119 ms | 同集合 4150 → 851 µs（fixture 4.9×） | 已由 bulk 拥有，不重复立项（其 6.3 面已关闭） |
| O5 | `evidence/o5-facts-layers.md` | 容量梯度 5 档；roomy 往返省 51,380 entry 与 20 次目录解析（54×）；sweep 四档 `p=0`；命中路径 ≤47 µs/请求 | 往返最多省 24.8 ms/10 请求，代价 3.69 MB 驻留 | 容器与 CP/Header 层维持；**更高层 store 与淘汰暂缓**（触发条件已写） |
| O6 | `evidence/o6-warmup.md` | 导航 15 µs/类、冷首问 1225 µs、发现 256 ms、放弃一页 18/42 µs | 预取 8 类服务 1 类 = +105 µs 且无回收（第二次请求仍 1 次物化） | **暂缓**：缺宿主轨迹需求与前台延迟目标（触发条件 3 条） |
| O7 | `evidence/o7-parallel.md` | 1/2/4/6 worker 3705/2773/2357/2119 ms（RSS 120→161 MiB）；w=4 的 90% 请求段等在 `take_front`、总账等待 0；取消臂真停 | −`take_front` = 236 ms（**不可读成可达收益**） | 类间并行已交付；窗口瓶颈归 bulk 未裁定架构决策；**W6b 暂缓**（缺宿主并发需求） |
| O8 | `evidence/o8-bytes-output.md` | W1 open 2.68 ms/2.9 MB（1.08 GB/s）、output 85 µs、未归因 46%（调用方发现）；W6a 编码 +2.8%、写盘 +3.6% | 编码上限 <5% | **编码改进否决**；增量摘要/mmap 否决；可信 digest 与 owned bytes 保持；CLI 长度固定点暂缓（触发条件=大输出单文档，≈0.85 ms/MB 未直测） |

**负向实验（八条全部真跑）**：O1 sibling 损坏不误伤、容量不足同语义指纹、96 sibling 零新增物化；O2 四态 store 同工作且无免费请求、未请求不解码；O3 损坏后缀三停止点；O4 两臂同结果 + 顺序/背压/范围反例；O5 内容变化/依赖补齐/损坏与取消不发布/满≠命中（8 组）；O6 "预取使后续免费"被证伪；O7 取消真停、跨 worker 同结果、窗口预留反例；O8 三档同结果、内容变化改变摘要、`elapsed_millis` 是唯一可丢字段。

**新测与引用的区分**：新测 = `investigation-{fixture,bcprov,bcprov-sinks}-raw.jsonl`（各配置 5 个独立进程、sink 档 3 个），复算脚本 `investigation-tables.py`；引用 = G0 基线、`add-parallel-bulk-recovery`、`add-demand-driven-core-results`、归档的 `bound-container-lookup`——**引用项的既有门禁本轮全部重跑**（`p5_container_lookup` 29/1ign、`p5_facts_cache` 13、`p5_shared_payload` 6、`p1_query_demand` 6、`d2_prepared_handoff` 9、`d0_demand_counts` 5、`bulk_recovery_*` 38，全 0 failed）。

**命令与结果**：`cargo fmt --all -- --check` 干净；clippy 零告警；`cargo test --workspace --all-targets --all-features --locked` **110 套 / 1592 passed / 0 failed / 18 ignored**；harness 自身 14 passed / 1 ignored。日志 `investigation-verification.log`、`investigation-workspace-test.log`。

**未确认项（不得读作已测）**：一次受信读取的单独耗时；store 的哈希/克隆/锁分量；W4 四臂的冷热顺序混淆；unit 内命中密度分布；本轮每格 5 个样本**不满足** bulk §14 的 ≥10 次交错判据；尾延迟与 `retained_bytes` 的 RSS 换算；CLI 长度固定点的直接测量。

**暂缓项与触发条件**：O5 更高层 store 与淘汰、O6 预热/预取、O7 的 W6b single-flight、O8 的 CLI 长度固定点——四者的触发条件分别写在各自证据文件的第 7–8 段。
