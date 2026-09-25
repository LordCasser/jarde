## 1. 冻结真实字段存储语料

- [x] 1.1 冻结 `p3-core/` 的最小永久 Java 8 fixture，保留字段/Fieldref descriptor 补丁、Code 前后不变的哈希、267 项原 class 验证执行与完整 jarde/JADX 阶段；另重放 `core-bcs/` 的 285 项，root 统一 census/fingerprint。原 380 项含 Z 样本保持边界证据。
- [x] 1.2 增加 B/C/S 极值、实例/静态写入、producer 正常/抛错、null receiver、每笔普通窄局部/常量对照；固定异常身份、字段旧值与调用次数。补 verified accessor 正例及无效 accessor 拒绝对照，不混入新 accessor 识别。

## 2. 在已证明写入位置表达截断

- [x] 2.1 在 `field_value` 仅对真实 B/C/S 字段写入及已确认 accessor 使用现有 Cast 表达必要整数截断；同型/widening/常量保留旧规则，不放宽通用 `meeting_position` 或调用参数。
- [x] 2.2 保持 receiver/value 的最终 at、先后和一次求值，实测静态、producer 抛错与 null 的 JVM 顺序；Z 与未知值继续拒绝。
- [x] 2.3 失败分支复用 `quoted_bcis` 修复已实证的 call@3/put@6 来源缺口；成功路径保留原 origin/预算/取消，默认/all 正文和 replay 稳定，无新的来源机制。

## 3. 完整类与 root 验收

- [x] 3.1 原样重编译执行永久 fixture 的 267 项与 B/C/S 核心 285 项，要求零引用、原 class/jarde 的字段值、调用数与异常逐项一致；JADX 当前编译失败只记阶段，不声称其运行。
- [x] 3.2 root 独立审读准入、Cast/访问顺序、来源及 verified accessor，重放真实完整类；复跑 field、array、return、required conversions、deferred 顺序相邻回归，Z 债务保持独立。
- [x] 3.3 root 统一 census/fingerprint、fmt、适当 Cargo 回归及 OpenSpec strict；完成证据后才勾选任务。
