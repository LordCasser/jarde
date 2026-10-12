# 当前保存对照的独立观察验收

root 实际执行 checker v1，因连字符 batch labels 与下划线内部 keys 不一致退出 1；Luna v2 仅修复映射，root 审查后实际退出 0。首次输出位于外置 root-v2 summary；本目录 invocation 又实际执行两版本，分别保留 exit 1/0 和完整原始 stdout/stderr。失败 --out 未生成 JSON，不能把它称为失败报告文件。

八份 debug/none、双 JDK 记录共有 224 条实际命令、448 份原始流；fixed/String/Integer 共 48 组完整生成类编译运行与冻结原类输出逐字相同，其中 Jarde 24、JADX 24。Jarde 已恢复确切 List<String/Integer> 声明与 add(local)，保留 remove(Object) 和真实 checkcast。两份合并控制共 24 组编译失败，无运行记录；handler 的物理来源缺口保持。36 个独立 oracle labels 的值、集合状态、异常与效果字段原样核对。

验收 scope 仅为保存对照完整性、原始 oracle、完整生成类和物理身份/map；product_acceptance=false、task1_2_complete=false。反例 SSA 归因、停止永久回归、内层实参与 null 的新恢复、最终 workspace/CI 尚待完成。focused v6 原始 receipt 证明旧三个公开 API 测试和 371 library 测试通过、CLI 冻结及 finally clean；当前新测试和后续产品修改未被 v6 验收。

captures/tooling 压缩档保留 receipt、原始流、源与 class、工具两版本；archive-pins 核压缩 byte identity。receipt 内原始外置路径未改写。固定 -g:none 本轮物理类没有独立旧 no-debug 编译锚；其输入、实际 javac flags、Signature/debug tables 和冻结 runtime oracle 已核，此限制在 summary 单列。
