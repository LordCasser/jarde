## 1. 固定可执行的错序回归

- [x] 1.1 将构造3项和调用/字段/数组/cast的18项审计整理为最小永久Java8 fixture，helper与runner保留源码；保存原编译输入及精确Code补丁，验证patched class的JVM合法性、hash和原/JADX/jarde完整类差异。补充已实测的整数除法/取余、getfield、arraylength及三种已有数组分配、嵌套内联对照与分支内直线案例，不能以手改恢复正文绕过问题。
- [x] 1.2 使用冻结输入的内存变体固定跨region/guard、未成功保存的失效局部、重复消费及无法声明类型的拒绝边界；证明原class确实可执行，不能靠非法bytecode验证保守拒绝。`negative-review/switch-core/`已实际验证跨join正常重复调用6项中2错，须正确呈现或完整来源拒绝。root统一冻结census/fingerprint。

## 2. 实现共享的求值位置与有名值保存

- [x] 2.1 在Builder既有SSA/指令索引与区域上下文上实现有界的内联位置证明，区分同一表达式依赖链与必须跨越的独立语句；未知位置不得默认透明，原有嵌套调用/实参保持有序内联。composite-boundary 15项及nested-producer-boundary 5项均由root独立重放通过；switch预生成消费者遵守拒绝决定。
- [x] 2.2 复用Declare/Local及现有类型呈现，在真实生产位置保存必要的值；成功提交声明后才发布绑定，不伪造JVM槽。调用、已认领字段、数组读取/长度/分配、整数除法/取余、普通cast和已验证构造共用同一绑定机制。
- [x] 2.3 将构造Site各身份归入同一次已初始化值，确保构造器位置、参数读取、唯一消费与失败生产者来源完整；禁止在消费者处重算或重复执行，禁止把uninitialized对象当作已声明引用。
- [x] 2.4 复用现有区域作用域及统一非槽名字分配，闭合普通直线、分支内直线和测试前缀；避免lambda、原局部及final字段名称冲突。跨区域/循环/guard未获证明时保留明确拒绝，不能生成不可见名字或改变执行次数。
- [x] 2.5 将证明遍历、绑定条目及新增声明纳入既有工作/IR/来源预算和取消/深度边界；默认与完整证据共享同一正文，失败提交不能泄漏未声明名字，commit/replay不重新决策求值位置。共享quote遍历按首次访问的SSA节点扣账并逐节点poll，名称与site来源逐候选/BCI扣账；低限测试和root代码审读的证据范围见verification。

## 3. 完整类行为验收

- [x] 3.1 用实际恢复的完整类重编译执行全部正面案例，对照返回值、对象身份、trace、次数及异常优先级；四份独立审计57项已证实案例必须真实恢复，不能统一变成引用来消除错序报告。补充内联对照和作用域正反例；`structured-scopes/`另12项必须实际保存并执行一致，不以未插入mark的普通if控制替代。root已重放预算版CLI的四组66项、复合15项、嵌套5项及内联/guard对照。
- [x] 3.2 root独立重放原始四份直线审计、structured-scopes的12项和composite-boundary的15项，检查零引用、完整javac和逐行运行结果；inline-controls的60项与guard-inline-core的14项保持原有序内联；nested-producer-boundary的5项保持两段独立效应与异常顺序。相邻回归与独立lambda RED的边界记录在verification。
- [x] 3.3 root统一验收Java包、语料census/fingerprint、fmt及OpenSpec strict，严格clippy的既存区域债务单独记录；实现边界与new消费者等后续独立任务见verification及审计目录。
