## 1. 固定行为与拒绝边界

- [x] 1.1 冻结 635 B 字符串 switch 原 class、JADX、jarde 的完整源码与10行 `-Xverify:all` 执行对照；验证 `string-switch/summary.json` 全部编译/运行退出0且两份 diff 为空，明确当前差距仅是源码形态。
- [x] 1.2 补充共享 case、fallthrough、default、抛错 selector、空串/Unicode 与非标准或篡改映射的合法 class；逐份记录 javap、hash/目标、原/JADX/jarde 编译与运行，并固定不可折叠样本仍保持现有正确行为。[共享 case 与穿透](../../evidence/java-syntax-2026-09-22/string-switch/variants/analysis.md)、[中间 default 空洞](../../evidence/java-syntax-2026-09-24/string-switch-variants/analysis.md)、[Unicode 与 null](../../evidence/java-syntax-2026-09-24/string-switch-unicode/analysis.md)、[额外 hash 消费和错误 bucket](../../evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/analysis.md)已有冻结事实；[root 最终 CLI 的三方整类重放](verification-2.2-to-3.1.md)又覆盖桶内副作用负例，正例四类与负例三类均按证明边界处理。旧基线里 MiddleDefault 的两层 Jarde 输出仅是实现前状态。

## 2. 局部形状证明与现有 switch 呈现

- [x] 2.1 对同方法 String `hashCode`、常量 `equals`、bucket 与 discriminator→最终 switch 映射建立有界完整证明；用碰撞、错误 hash、重复标签、额外前驱/效应的正反单测验证准入与拒绝。
- [x] 2.2 在现有区域构造中让成功证明的组合共同拥有两层分派并保留最终 arm/默认/共享目标/fallthrough；用源码测试确认只输出一个 `switch (String)`，失败证明仍输出原有两级结构。
- [x] 2.3 将现有 switch AST 的 enum 标签 sidecar 收敛为枚举/已验证字符串的封闭展示标签类型，整数 `keys` 仍保留原始证据并复用 emitter；用 Java 8 编译测试确认普通整数与 enum switch 不变、字符串字面值合法且无重复执行的 hash/equals 语句。
- [x] 2.4 将选择器与两级分派的来源、预算、取消和默认/all 正文一致性闭合；用来源覆盖、低限停止与完整证据测试验证没有丢失或伪造 BCI。[验收记录](verification-2.4.md)

## 3. 完整类与独立验收

- [x] 3.1 原样重编译执行 1.1/1.2 的正面完整类，对照原 class/JADX/jarde 的每行结果、调用次数、null/抛错顺序，并验证未证明样本保持当前执行结果。[root 独立重放](verification-2.2-to-3.1.md)确认四个正例完整类共 51 行原/JADX/Jarde 一致、三个拒绝变体的 Jarde 执行不变，且新 Jarde 折叠了 JADX 未覆盖的 default 空洞。
- [x] 3.2 root 独立审读局部证明与字符串标签不变量，重放三方完整类和整数/枚举 switch 相邻回归；再运行适当 Cargo、fmt、OpenSpec strict 与语料 census/fingerprint，保存验收结论。[独立验收](verification-root-3.2.md)确认功能定向全绿，记录跨在途变更的指纹 60 项未登记与严格 Clippy 7 项既存 lint；不宣称整仓门禁全绿。
