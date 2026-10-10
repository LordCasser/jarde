# 局部类型证明：root 架构审查

root 已阅读当前 decide_types、SlotUse/reuse、SSA definitions、declaration/调用呈现、reference_lifetime_type/array_of_value/constant_of_value，以及 JVM constructor token/frame writes→SSA instruction definitions 周边。真实诊断与独立接受见 `openspec/evidence/java-syntax-2026-10-11/cf12-real-ir-diagnostics-root-v1/`。这次审查接受 v3 的有限证明方向，不是生产测试或产品交付验收；前片自身 CI/clean 关闭前不应用生产。

v1 null proof 复用了过宽的 reference_lifetime_type：Named frame 无须 producer definition，Unknown frame 可能沿 Load/Store/Duplicate/数组读取追溯，不能满足本片直接来源边界。v3 改为真实 Instruction 上的直接 String/Class 常量、准确返回/字段描述符、allocation/array creation；不穿过 Phi/Load/Store/Duplicate。所有非 null 写必须同一准确 Reference，至少一写非 null，完整 reuse path；Object fallback 和 callee 需求不构成证据。

直接 new String 的 stored value 可以由 `<init>` 的 alias initialization 定义产生，不能把所有 void invoke 都拒绝，也不能泛化接受任意 Named frame。root 核 frame.rs constructor_call 的未初始化 token、owner 匹配和 convert_token，并核 ssa.rs 将实际 frame write 记为同一 BCI Instruction。v3 仅准 Special/<init>/void + 同一 SSA 指令的 Ref(Named) 与 constructor owner 精确相同，不遍历 copy/phi；真实 CF12 StringBuilder stored 已显示此种定义。应用时 owner spelling 统一复用现有 spell_reference，避免另一份 replace 逻辑。

char 仅直接 C call/field、i2c 或读取 entry C 参数种子；全部写均为准确 C 或 0..65535 的直接 int literal，至少一个 C 种子。算术/iinc/未知copy/phi/越界不能准入。原参数→boolean→guard/multi-return/constructor专用证据仍优先，新规则只补既有map的普通局部尾部决策，不改JVMframe类型或调用转换门。

同样的字节码可以来自原 int 局部持有 char producer。root 已核 invocation_argument 的 descriptor-selected widening 必须显式 cast_argument，故 append(I) 不应因新 Char 声明误选 append(C)。private CharProducerIntOverload 真正输出46与误选后`.`可区分；原 Java 边界12命令/4运行腿仅接受原始输入，候选重载行为、构造写、reuse身份和负例尚需真实工具链与永久证明测试。

private v3 permanent test 缺 trusted ArtifactSubject，不能据其 physical origin 断言宣称已验。v4 已补同次 analysis.report method/environment 和同一 snapshot PreparedClass 唯一 locate_method 返回的真实 ordinal，root核其与facade读取规则一致。full_usage-1 仅测末期原子停止，不证明新规则内预算门；后续必须用真实 SlotUse/SSA 在内部 helper 上实际测 charge/Stop BCI 与取消，再核无部分正文/map。

`private-product-patch-check-root-v1` 只证明完整 v3 diff 对当前源码 `git apply --check` 实际exit0。private-candidate-luna-v3/v4 原稿完整保留，未编译、未应用、未视为通过。本片的条件fallthrough、嵌套常量名及其他债务不混入该map改动。
