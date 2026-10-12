# 六份旧产品基线的 root 独立汇总

root 审查 Luna v2 后复制只读工具并通过 record.py 实际执行，`invocation/execution.json` 记录退出码 0。`summary.json` 核对六个既有 capture 的 166 条实际命令、332 份 raw streams、28 个精确 oracle labels、源/工具/冻结 CLI SHA，以及 24 个报告的物理 class BLAKE3、length、method binding 与原始 map origins。无调试三家族 javap 均没有 LVT/LVTT/LineNumberTable，泛型 Signature 保留；双 JDK、debug/none 原始 oracle 分别相同。

独立 String/Integer 四份 capture 共 32 组 JADX/Jarde 完整类编译运行成功且 oracle 相同；旧 Jarde 仍是 raw List，不证明泛型恢复。合并控制两份 capture 共 24 组编译失败、无 runtime，4 个每模式的 handled map gaps 保留。不得把失败分组数当通过或按单方法编译冒充完整类。

当前仍 `product_acceptance=false`、`task1_2_complete=false`：停止控制和全部反例 SSA 归因未闭合。v1 私有准备未执行；v2 修正 oracle 五字段/四分隔符解析、精确 labels 集合、预先 pins 校验，失败的更早 observer 已分别在基线目录保存。

`handler-gap-audit-luna-v1.md` 是独立只读架构审计。root 已在当前 region.rs 核实 loop_body 清空 own_try 和 header-tested gate 缺少 named-catch 验收；报告提出复用原有精确 catch-edge 校验，并未实现或构建。它不是当前泛型补丁的附带修复。

外置原始输入路径在 receipt/pins 中原样保留；仓库复制不改写物理路径或 receipt。前次 CLI 和源码观察仍保留。
