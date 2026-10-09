## 1. 冻结与消费位事实

- [x] 1.1 冻结 acceptance.md 中正例/控制族与既有 CallHold/ExceptionHold/BoundOverload 的真实 Corretto8/OpenJDK23 × debug/no-debug 输入、原/JADX/baseline 完整结果；核对输入源/jar、实际命令、Probe、CLI/JADX libs hash 与所有分类，验收 GC-01～GC-10。先前 exploratory 不代替四腿，保留失败历史。
- [x] 1.2 在既有同次 Code/SSA/最终 Program 路径提供需求条件化 opaque AST 与精确 invoke receiver/arg/result 全消费事实；按物理 caller/BCI/opcode/目标及实际槽位唯一交叉核对；通过数组/宽槽/多use、origin多义/折叠和预算取消定向测试，验收 GC-01、GC-02、GC-08、GC-10。

## 2. 关联声明与类型证明

- [x] 2.1 在现有 method→field publication 边界建立有限关联 staging、callee-first 依赖证明和最终全 incoming-use 验证；保留原始头/正文并一次原子提交或关联回退；通过逆声明序/两层以上relay、失败incoming、独立成员不退化与依赖环控制，验收 GC-03、GC-08、GC-10。
- [x] 2.2 实现实际 class/method binder 环境下的直接 T/T[]、bounds、多参数/宽槽与直接 callee method-formal 代换；扩展 empty/void/实际invoke返回 caller 证明，不伪造 Parameter 候选；通过泛型反射中 GenericDeclaration身份、marker/数组身份和完整类重编，验收 GC-01～GC-05。
- [x] 2.3 对 Object() 后调用字段赋值及已完整恢复的普通 catch 构造器，独立证明 InitRecord、全部参数/调用消费和异常顺序；不放宽旧 constructor 空体/隐藏规则；用冻结 CallHold/ExceptionHold 四腿与异常marker对照验收 GC-06。
- [x] 2.4 在最终 header/AST 实参类型下证明封闭同类 overload 目标；唯一时保持直接调用，必要时仅投影已证无运行时检查的上转型；完整 BoundOverload 四腿选择number、plain overload及未知集合拒绝、物理origin/source map核对验收 GC-07～GC-08。

## 3. 独立验收与交付

- [x] 3.1 对冻结全部四腿运行候选完整源码重编、-Xverify:all、行为/异常/调用目标与结构化泛型反射；root独立核对CLI/源码/输入/Probe hash、实际stderr和失败控制，不借原jar、不删成员；逐族填写结果而非将CLI 0当成功，验收 GC-01～GC-10。
- [x] 3.2 回放既有23字段族、80构造输入、64raw receiver输入，保持各自完整编译/行为/已验收反射范围；完成 fmt、CI白名单同口径clippy、两固定seed、显式ignored P3/functional-constructor/bound-receiver与strict OpenSpec；保留首轮失败及修正日志，验收 GC-09～GC-10。
- [ ] 3.3 root审阅最终架构与边界，更新本片verification/handoff和71单元局部账本，提交推送main并确认最新HEAD实际CI四job成功含真JDK25；核对工作区/分支占用、清理共享Cargo并记录磁盘；不得提前勾选交付或宣称整个generic单元追平。
