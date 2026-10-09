# 计费工作量固定值更新 v1

本次固定值更新依据已保存的 `root-p5-record-v1` 记录器输出和旧固定值失败输出。普通构造器验证器现在接收共享的 `Budget`，并为实际读取计费；此前普通路径使用 `None`。以下是实测计费变化，不代表性能改善。文本、结果以及其余七个计费维度均保持不变。

| 用例 | IrItems：旧值 → 实测值（增量） | AnalysisSteps：旧值 → 实测值（增量） |
|---|---:|---:|
| flat-mixed | 2473 → 2497 (+24) | 1360 → 1501 (+141) |
| nested-mixed | 4298 → 4322 (+24) | 2093 → 2319 (+226) |
| two-origins-one-identity | 2058 → 2082 (+24) | 1121 → 1244 (+123) |
| many-method-class | 21278 → 21384 (+106) | 10609 → 11784 (+1175) |
| damaged-tail | 2058 → 2082 (+24) | 1121 → 1244 (+123) |
| deep-expression | 3213 → 3213 (+0) | 1172 → 1366 (+194) |
| **六个用例合计** | **+202** | **+1982** |

逐方法两条 arm 的实测固定值分别为 `DIRECT_ARM` 和 `SHARED_ARM`：两者均从 `IrItems=35378, AnalysisSteps=17476` 更新为 `IrItems=35580, AnalysisSteps=19458`，恰好等于上述六个用例增量之和。旧值和实测值分别记录在 `root-p5-old-pins-v1` 与 `root-p5-record-v1`。

仓库 class fixture census 从 `(1055 classes, 4532 Code bodies, 459 handlers, 2711 branch targets, 8 subroutines)` 更新为 `(1063, 4616, 463, 2711, 8)`。新增的八个 canonical class 文件包括两个完整正例类的双 JDK 编译结果，以及两个 handler 控制版本的双 JDK 编译结果；共增加 84 个 Code body 和 4 个 handler，没有增加 branch target 或 subroutine。
