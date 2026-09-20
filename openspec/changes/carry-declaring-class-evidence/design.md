## Context

见 [proposal](proposal.md)。`engine::raw_facts` 持有 `read.header.facts.this_class/access_flags`；同处的 FrameDeclaration 已取得 `this_class`，但公开只读 MethodDeclaration 只保留 member flags、name、descriptor、slots 与 identity。`facade::recovery_facts` 因此没有设置 `MethodFacts::with_declaring_class`，而 `declaration::plan` 对普通成员必须有该事实才能确定 declaration form。

这条诊断是声明 envelope 的证据缺口，不能仅凭出现 10,720 次就归因于 fallback 的 50.9% 或纯注释的 17%。它与字段/new/handler 的证明失败分别统计。

## Goals / Non-Goals

**Goals:** 交接本次已经取得的两个最小字段，保持只读、同源、无额外扫描。

**Non-Goals:** 不把所有 ClassFacts/CP 再复制进 MethodIr，不新建按需声明 provider，不扩大 Body 闭包，不解决所有类级 metadata。

## Decisions

### 1. 扩展既有 MethodDeclaration

在 raw_facts 构造 MethodDeclaration 时保留 driver 的 raw `this_class` 与 class `access_flags`；沿用其物理 method.owner 身份和内部构造路径，不从请求 owner 的显示字符串反推。字段使用 reader 的 JVM 字节表示，展示转换不参与 identity 比较，不采用 lossy 字符串做绑定。新增只读 getter，仍无对外可变或任意构造 MethodIr 的入口。

不复用 FrameDeclaration 的私有字段作为恢复接口，因为那会把 Frame 的实现载体暴露到 java 层；不重读 Header，因为已取得的信息不需要再付一次 I/O、解析和 Budget。

### 2. facade 适配到既有声明事实

恢复适配设置现有 DeclaringClass 的名称/flags；现有 declaration rule 继续判定普通实例/static、interface default/static 及 initializer。非法 Java 名称只影响安全显示及 syntax 状态，不能改写物理身份或声称合法源码。如果现有 String 入口无法无损展示 JVM 名字，只在该边界补充 reader 已有的安全转义表示，禁止另建名称体系。

未产生 MethodDeclaration 的停止路径保持 None；没有 Code 的 abstract/native 继续保留当前分析适用状态，不为了提供内容制造空 body。本 change 不改这些路径的 Produced/Stopped 策略。低层恢复 API 仍可对缺失证据给出既有拒绝，公开 Engine 的同源事实不得丢失。

### 3. 验收以证据和读取范围为准

普通类实例/static、interface default/static、构造器与 `<clinit>` 通过公开入口检查 DeclarationRecord；特别验证 class flags 与 member flags 不能混用。不同 snapshot/loader 的同名定义不能串用 facts。针对普通无 accessor 方法，Header/Body 读取次数保持修正前基线，源码原始读取字节不增加；新增小字段占用遵循已有派生物计费，不用“不增加 I/O”冒充零内存成本。

对低层 missing-facts、早停/取消、无 Code、非法名字保持负向回归。删除 facade 的声明类适配应使公开入口测试失败，防止只测试 `with_declaring_class` builder 而再次遗漏闭环。

### 4. 复用与层次

复用 reader Header、MethodDeclaration、DeclaringClass、现有 declaration rule 和安全显示；不存在需要外部库承接的缺口，不新增依赖、许可证或版本选择。class flags 是 parse 事实；它不证明 dialect 合法、成员 runtime 解析成功或 JVM verification，通过交接也不自动提高 source recovery 的质量。

## Risks / Trade-offs

- 同名类串用 → 以同一读取得到的物理 identity 绑定，不用 class name 做缓存主键。
- envelope 诊断消失被当作质量大幅提高 → 独立记录 declaration 变化，body content/quality 另测，不提前承诺 corpus 收益。
- 向 MethodIr 塞入全部 Header → 只传 class name/flags，已有成员字段与身份继续复用。

## Migration Plan

扩展只读 declaration 和 facade 适配，补公开入口对照，再更新“类级声明事实尚未交接”的支持说明。R8/R9 回归及源码映射继续在同一候选上运行；不修改其它 class metadata 的支持承诺。没有存储迁移或兼容层。
