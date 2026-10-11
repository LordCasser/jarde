# CF11 TestNestedLoops2：同次 IR 的实际 ForHeader 首拒

Root在593fbcc0b主线实际运行一个临时库test，原878-byte javac23 class经Reader/canonical/SSA/NormalFlowView/recover。同次10节点/12边/28SSA指令/19trace；完整4707输入恢复，默认栈、5GiB free/target1GiB一秒守卫未触发，target peak156280445bytes、clean0/target不存在。生产源码去除logger/test后逐字节恒同。实际调用与stdout/stderr在invocation；source pins、class/javap绑定和原raw见execution。

外层header4：唯一latch41，init3写slot3，update41为iinc、transfer44，准确两输入phi及前置门通过；随后在条件Invoke List.size@6明确拒绝，返回Ok(None)。此前静态候选由本次真实trace证实。内层header26在当前preheader末尾不是j初始化store而拒绝，不应强制成for。最终恢复仍structured且无fallback，外/内while，十节点均claimed。

private v1/v2/v3未运行：root审查先修preheader引用类型、真实Boolean返回标记、两个Option格式。实际运行v4；原稿patch/pins保留。测试调用Some(false),true与原方法flags/descriptor匹配；不借错误false的旧诊断。没有实现生产白名单扩展或接受任何新for。

下一方案保持现有init/update/phi/invariant/owner判据，复用header_tested_loop已有test_is_pure与Builder条件恢复，让已证条件中的非local-writing producer可留在for头同一求值位置；新增effect/throw控制后再按OpenSpec实施。泛型raw List投影独立记录，不混入该改动。整个CF11与71/612计数不变。
