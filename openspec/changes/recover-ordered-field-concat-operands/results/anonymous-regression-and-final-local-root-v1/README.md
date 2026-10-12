# Anonymous concat regression and final local gates

2026-10-12 root 实际核验；不代表 fresh 全工作区、自身产品 CI 或交付完成。278 个归档条目已逐一重读核字节，capture 的长度/SHA 见 manifest.json。冻结 CLI 只保留 receipt pin，不重复存放可执行文件。

## 全工作区失败的归因

首轮 full-workspace-root-v2 在第一 seed 的第 6 个命令失败：`class_source` 为 101 passed / 1 failed，其余两个已构建 target 因 Cargo fail-fast 未执行。cardinality mismatch 是失败后的收尾诊断，不能当成测试覆盖已经完成；该次全部 raw 与 finally clean 保留。

失败的 `anonymous_superclass_projects_the_local_declaration_initializer_shape` 仍要求 `append("body:").append(local1)`。父项与当前 CLI 的同一冻结匿名子类报告证实，head 0/25 此前均因真实实例字段读而报 `jre_concat_interleaved_effect`；新字段许可分别恢复成 `"body:" + local1` 与 `super.render() + ":" + local1`。不是既有已接受纯链或 field-copy 链的文本变化。

新增永久测试从同一 payload 的真实 event 参数和 areturn 值反追，实际路径为字段 13 → append(String) 16 → toString 19 → event 22，以及字段 45 → append(String) 48 → toString 51 → areturn 54。它检查紧邻的 receiver Load、完整指令/Phi 消费唯一性、同块、完整有序异常表覆盖、field/concat 报告，以及字段 Direct / append Derived 的同物理方法来源。测试不以猜测 BCI 选择链。原断言只在取证和运行对照通过后更新。

## 完整类运行与保留的独立缺口

comparison-root-v2 实际执行 29 个命令；原 class、原 Java 源码和 JADX 源码在两个 JDK 上输出相同。它将 Jarde 三份物理类文本同时编译时，父项/current 各两个 JDK 均失败且无 runtime：独立匿名子类构造器把捕获字段赋值放在 super 前。这个已有缺口没有随本片修补；与账本中捕获匿名伴生构造器的既有债务一致。

projected-root-replay-root-v1 另执行 8 个命令，使用每个 CLI 实际生成的完整 AnonymousSuperArgs（含匿名体投影）和 Base 两份源码；每份字节均等于原 CLI JSON report.text，没有修补。父项/current × JDK8/23 的四个编译、验证运行均为 0，stdout 与冻结原 class 的两行字节完全一致。它证明投影完整类的拼接变化无行为回退，不关闭独立物理子类文本的构造器缺口。

Luna v1/v2/v3 脚手架问题与未执行材料均保留。v2 checker 的字符串 JDK path 读取错误由 v3 checker 修正；root 独立执行后接受观察。v3 projected runner 误选 original 源码，未执行；实际执行的是 v4，精确选取各 CLI 的 generated source pins。编译失败从未计入成功。

## 最终本地检查

final-check-root-v4 的 fmt、当前 CI 精确 Clippy、lib 390/0/0、p3_patterns 92/0/0、CLI build、全局 OpenSpec strict 及 finally exact cargo clean 均为 0。4854 条完整源码闭包前后及当前字节一致，default stack，5 GiB / target 1 GiB / 1 秒进程组守卫，target absent。root accept-v3 额外要求新增匿名字段值流回归具名成功。

receipt SHA `3612d48e7951552b70e61bfa36fa083582316a7f469394e05e7e435a934f5597`；CLI 62276032 bytes、0555、SHA `f991644eb87cef29df1509b270ba48d7c5d26ba72f46a5fae3271fc4528e3622`，与此前已接受的父项/字段完整运行矩阵 CLI 逐字节相同。最终源快照也与该 focused manifest 逐项核对。

较早 Clippy int_plus_one、root 新测试错误调用 MethodFacts.identity、单项预备测试 unused tail warning 均保留；当前通过不覆盖这些失败记录。单项预备测试曾执行 rustfmt，闭包变化只在该测试文件，故仅作为取证预备记录，不冒充最终源码验收。

归档时 fresh full-workspace-root-v3 从 target absent 重新执行双 seed 全 inventory；随后在第60个命令的 TWR 历史整类golden失败并 finally clean。该运行不在本 capture 内，也未接受，不得拼接旧失败批次。精确产品提交推送和自身 CI 尚未完成，未增加 71/612 或整体 CF18 完成数。最新状态见仓库 handoff.md。
