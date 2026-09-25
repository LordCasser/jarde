## 1. 固定动态参数与原目标的完整类反例

- [x] 1.1 将已实证的 String/Object 重载、数组动态限制及 primitive 同型对照整理为一份最小 Java8 永久 class；helper/runner source-only，保存 javap/bootstrap/hash 和原/JADX/jarde 完整类三方基线，不因其它形状失败而裁剪生成正文。
- [x] 1.2 补充动态 String→实现 Object、无捕获实例 receiver、constructor 的真实调用与 null/错误类型对照；对 bound-null、真实 SAM 返回窄化、boxing、未知引用关系分别记录正面或明确拒绝边界。保留 return-probe 的 raw Supplier 返回 Integer、typed caller 自身 checkcast 失败的成功对照，不能把 instantiated String 返回误当新检查。

## 2. 在现有函数式计划中保留类型阶段

- [x] 2.1 现有 Plan 已保留擦除/动态/实现参数与受限返回证明；9/9 lambda 单测核对 Object→引用/数组的动态检查、同型/Object 上溯、未知转换拒绝和 raw Supplier 返回控制。root 另跑 immediate functional receiver 2/2；Builder 尚未消费新事实，错值样本仍待 2.2/2.3。
- [x] 2.2 用现有 Lambda/Local/Cast/Call/New 构造受支持的显式适配，擦除参数作为声明，动态检查先于精确实现类型；root 原样编译原 class/JADX/Jarde 完整类，String 重载、数组和动态 String→实现 Object 在 27 行执行对照中一致。
- [x] 2.3 MethodReference 只在无需参数适配时保留；无捕获实例/constructor 的参数与 receiver 顺序由完整类异常优先级与调用计数验证。bound receiver 缺创建阶段证明时明确拒绝；旧手工测试的 `Object local1::run` 非可编译源码，已改为拒绝断言。同型 primitive/generic return 与非 void→void SAM 控制通过。
- [x] 2.4 复用现有命名、来源、预算/取消与拒绝路径，验证 essential/all 正文相同、真实 site/CP/capture 来源完整、低工作/IR预算停止且无半适配正常产物。带捕获 BCI 4 与 site BCI 5/CP #13、两个正例内部 AST 阈值及 bound-null 拒绝路径均由独立 [propagate-value-rendering-stops](../propagate-value-rendering-stops/tasks.md) 收口；入口预取消 IR 表误分类另案。

## 3. 主代理独立验收

- [x] 3.1 root 原样重放初始 6 项和扩展受限正例，比较值、目标、检查、调用次数与异常；候选手写 lambda 仅作设计证据，不能算功能通过。
- [x] 3.2 root 检查无新增泛型/类型解析机制，复跑 lambda/method-reference、调用实参、cast、deferred-order、来源及预算相邻回归；helper 命名与捕获扩展债务单列。完整结果见 [verification-root](verification-root.md)。
- [x] 3.3 root 统一验收 Java 包、永久语料 census/fingerprint、fmt、严格 clippy 与 OpenSpec strict；严格 Clippy 的 48 项阻断、旧 reader census 钉值和已通过的定向测试均如实记录于 [verification-root](verification-root.md)，没有把门禁失败记作通过。
