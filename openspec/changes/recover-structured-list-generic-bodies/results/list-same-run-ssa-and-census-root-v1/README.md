# 结构化 List 的同次 SSA 归因与 census 回归

Root 独立验收器 v2 实际 exit 0，接受 18 条同次事实事件，覆盖组合控制的 10 个方法、固定正文的三个请求选择、独立 String/Integer、内层实际元素和两个 null 正例。逐冻结 class 的真实 Code 核每条 BCI/opcode；SSA/effects/operations 数量、唯一性、实际 local 读写及全部 def/use 引用一致。诊断只临时插入同次 IR 已存在的出口，未重跑证明或改变预算/决策；完成后恢复的 4822 来源 pins 与 focused v9 全闭包一致，只有 report.rs 在诊断期间不同。五个公开目标测试实际成功，finally clean 0、target 不存在。

实际归因：rawAliasEscape 的 slot 1 首读 @0 只交给 store local 3 @1，不能当成已证明 List receiver；reassigned 的 slot 1 @1 真写入 SSA value 8，@2 读取的是该新值；handled 有真实 [0,47)→50 handler 和 Caught SSA；indexOverload 区分 remove(Object)@2、remove(int)@11 及 add@19 的 receiver/实参；castBoundary 保留真实 checkcast@26/60。两个 null 正例中 aconst_null@77 的值只供 add@78 的 Stack(1) 实参使用，不能借此删除任意物理 cast。固定和其他正例完整读取清单、接收者关系、物理 cast 见 summary 与原始 facts。

SSA 事件没有泛型 Signature，因此另以已冻结同字节 controls javap 补足 wildcardNull 的 `(List<+Number>;List<Number>;)V`；不能由 null 数据流猜 variance。API拒绝是独立证据，结合有限精确 List<S> 门和真实槽/用途事实归因。之前原/JADX/旧Jarde debug/none 双JDK完整类基线和最终三方对照的编译、状态、异常、效果顺序保持原记录；24 控制编译失败均无 runtime，不改写为成功。precancel/lateStop 的准确预算及事务边界在 focused v9 与 final lateStop 原记录中，不能把这里的插桩记录当成交付 CLI。

Reader 真实预检 exit 101，实际 (1140,5018,474,3141,10) 与旧期望不同。Root 用已独立接受的 classfile decoder 逐新 7 class/23 Code 清点得到 +7/+23/+1/+93/+0：六个嵌套正文各 8 分支，组合控制 45。Luna 保存 javap 逐目标多重集一致。最初私有 scanner 的 83 由遗漏三字节指令长度造成；不能按猜测改预期。Root 只更新此准确增量和当前 30 个夹具登记，原 2134 记录不变，总 2164。

永久新增四个真实 IR census 测试：完整多 block/Phi 和全部读、漏/重复/错 BCI/错 opcode 拒绝、未知 invoke 身份保留真实已读而不声称 receiver、真实参数写及 handler 拒绝。第一次定向 v1 全部失败于 helper 的 InputBytes=0，记录保留；补齐现有 IR 测试的有界输入/读取/输出/头/方法/深度预算后，定向 v2 4/0、reader census 1/0、fingerprint 5/0/1 全成功，finally clean 0，4822 格式化后来源闭包稳定。生产 report 字节除新增测试模块外与诊断前完全一致。

同次事实 root 验收 v1 自身因把 Code 顺序和 SSA block 顺序直接比较而失败，原 raw 和源码保留。v2 按 BCI 排序但保留多重性；另按实际 ExceptionHandlerFact 字段核范围。此归档证明 1.2 所需归因及 2.1 的永久测试，不代表 fresh 全 workspace、自身 CI 或主线已交付。

captures.tar.gz 保存诊断三次真实执行、root 两次验收、独立 reader 增量和定向测试两次执行。tooling.tar.gz 保存诊断插桩及恢复字节、离线解析器/facts、分支清单、原测试候选和登记补丁。原 receipt 中绝对路径不改写；若重放历史脚本，按记录恢复 `/private/tmp` 下对应目录，并保留其已固定 decoder/guard/source-closure 依赖。controls-javap.txt 是 Signature 补充原始输出。所有 archive 与汇总 SHA/大小见 archive-pins.json。
