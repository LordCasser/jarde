# TWR regression attribution and final local gates

2026-10-12 root 实际验收。243 个归档成员已从最终 gzip/tar 重读，逐项核长度和 SHA；capture 2,652,620 bytes，SHA `04a84b3e950543454959c55aa9d3192694e1b75915842bee1df3d392ae08833c`。manifest.json 列出完整成员；冻结 CLI 仅保存 receipt pin，不重复存放可执行文件。

## 保留的全工作区失败

fresh full-workspace-root-v3 在第一 seed 的 command 60 失败：TWR target 3 passed / 1 failed，同批另外三个 target 共21 passed。源码闭包前后相同，finally exact cargo clean 成功、target absent。完整 execution、metadata 和124份原始输出单独放在 `failed/full-workspace-v3`；它不是完整工作区通过证据，不能拼接较早运行补齐缺失批次。

## TWR 完整类与文本对照

TWR differential-root-v2 的23个命令全部成功。冻结原class各JDK运行一次；原TR.java、JADX、父项Jarde及当前Jarde的完整源码分别在Java8/23编译运行，八个源码lane均成功，stdout逐字同原。生成源码字节等于实际JSON report.text，没有修补。root另核源码/输入/CLI pins、原始输出及编译参数；receipt SHA `9757773708e01bf0983bdd47e46fe5ebdc35df7d106030adce3398dfb8b7e3f1`。

精确method文本差异仅为 `use()` 与 `close()` 的直接实例字段拼接由Builder恢复成+。`one()`、`two()` 完整资源方法逐字相同。永久测试保留历史TR-rendered输入，只精确替换这两段预期，其余文本仍比较；原TR.class和历史渲染文本也在本capture冻结。额外field-copy和Unicode测试均通过，不修改它们的预期。私有TWR prep v1的问题在执行前被发现，实际运行v2；两版材料都保留。

## 当前最终局部门禁

final-check-root-v5 的fmt、当前CI精确Clippy、lib390/0/0、四target实际4/0/0（TWR）、3/0/1（field-copy）、4/0/0（Unicode）、92/0/0（p3 patterns）、CLI build、global strict与finally clean全部为0。4854条源码闭包前后/当前字节相同；八份修改源快照按该manifest逐项核对。default Rust stack、5GiB可用/target1GiB/1秒守卫、target absent；root accept-v3实际接受具名proof/停止/匿名来源回归。

receipt SHA `7fd0a7794c7be065f132d4b96f0a4adaf5dacbacb26e823e3c03dbc90dee854b`。CLI 62,276,032 bytes、0555、SHA `f991644eb87cef29df1509b270ba48d7c5d26ba72f46a5fae3271fc4528e3622`，逐字同已接受的父项与字段完整运行矩阵CLI。此归档不替代矩阵，参见相邻 integrated-comparison-root-v1 与 anonymous-regression-and-final-local-root-v1。

私有archive prep v1因额外future-workspace门槛、错误源路径与可执行脚本文本过滤而未执行；v2经root审阅修正后实际生成并重读，root再次压缩后重读所有成员。两版准备材料也在capture保留。

归档时fresh full-workspace-root-v4仍活动，本capture不读取或包含它。完整双seed门、自身产品CI、提交推送和干净main尚未完成，父项6/8、子项4/6；不增加71/612或整体CF18完成数。最新状态见仓库handoff.md。
