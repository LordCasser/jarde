# 完整产品验收

产品 `2684b4ee79a263ceb851d73c84e6ce4499dbd4b4`，CLI SHA `56ed2f31f824187b178082d1045788e5ba09ea8f9ab4e1fe6bffeca37db8efa8`。fresh native 实际 388 targets、102 batches、3794 通过/0 失败/98 既有忽略；48 精确门及九项 CF07/CF11/finThrow/nestedFin 回归各执行一次。default stack、每秒 guard、峰值 847254836B，finally exact clean，target absent。3240 build 与 5193 native 来源并集 5200 路径逐 Git blob/bytes/mode 绑定产品和冻结 CLI。

自身 [CI 39178836424](https://github.com/LordCasser/jarde/actions/runs/39178836424) 全部四 jobs 和 52 独立官方 ZIP 步骤接受，含 Clippy/MSRV1.88/专项 JDK/供应链/fuzz。双 seed 各 388 targets、3794/0/98，93 精确门（含 13 inherited extra-critical）各一次，全部 3892 具名结果一致。没有从 aggregate log 拼造步骤。

`capture.tar.gz` SHA `eab834fed317de7432948b5289799f41de2f0229d526186c8fd98d2578e37b65`，831 regular payload、147 目录记录及内嵌库存，root 全部逐字/mode 复开，见 `root-review.json` 与 sidecar inventory。包含真实 native/绑定/周边回归、全部原始 CI 证据、接受器、下载失败和重试工具。

下载过程与 CI 成败分开：v1-v6 是真实 API EOF，v7 原始 ZIP 请求 EOF；v8 的逐字复制尝试被严格接受器因流路径仍指旧目录而拒绝，未放宽检查。v9 重新实际采集所有 job/source 和官方 ZIP，接受器原样通过。运输层仅串行、有界三次重试，各次 raw 均保留，最终原件逐字复制，无日志重建；没有重跑 CI 或测试。具名观察器首次在接受器 stdout 尚未保存成 review.json 时读取失败；保存逐字 stdout 后复审通过，未重跑目标。

后续主规范、账本和 handoff 提交仅交付文档，不冒称该文档提交另有已接受产品 CI。目标方法未调用，四 API 来源修正与原完整编译复用边界见上一层 README。
