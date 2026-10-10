# 实际异常边反例与范围

root 使用固定 JDK23 --release 8 编译 ExceptionIfJoin，352 bytes；实际 javap、源码、argv、stdout/stderr 和工具 SHA 均在 exception-join-case-root-v1。没有执行此反例目标代码。

同一次 analyze_method_ir→region::recover 表明 Try 内循环的 If@12 有 join27，但 then 是 Fallback(ExceptionEdge block15/handler0)。同一个 canonical block 的 SSA 是 invoke15、iinc18、goto21，全出边准确是 Normal→27 加 Exception→36。新 helper 对这个实际 arm 返回 None；没有伪造 Straight 或图去逼出预期。方法继续按既有规则引用字节码，不加 If derived21。

Luna 私有稿先假定该 arm 是 Straight，root v1 实跑失败；修正为实际 Fallback 后，v2 又揭示整个方法因 local1 穿越 fallback 而被引用，source map 只有 block leaders，physical21 没有 span。这个来源覆盖缺口是既有拒绝路径的独立债务。v3 保留真实形状/两边断言与无 If 来源，1/0/0 和全部 gateway12/0/0 实际通过。失败和原稿都不覆盖。

对于合法 unconditional goto，raw CFG 只能有一个普通目标；额外 Exception 边来自同块受保护 may_throw 指令。无需制造非法双 Normal 图。当前范围不修 Try/loop 的 catch 传播，也不扩大 fallback 来源保留规则。后续若要恢复该结构，先从真实 Frame 路径另立 spec。

只读Frame审计与真实树一致：region.rs:2156 protected 设置 own_try；2039 loop_body 构帧在2059清为None；10544建loop body后，仅10558～10580几个已证finally/guard布局恢复，不包含普通named catch；2084 arm传递已为空的owner，2973～2990异常edge gate因owner缺失拒绝。7094 edges_accounted_by_catches可核已有owner下真实边和handler范围，但宽泛复制owner是否安全尚无证明。这个“外Try包Loop”与已有“Loop体内Try”债务不同，另列处理，禁止直接拷贝own_try强行通过当前反例。
