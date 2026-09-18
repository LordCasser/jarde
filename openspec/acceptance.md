# 架构验收与阶段映射

来源：[最终架构 §20.3](../JVM_Rust_Engine_Final_Architecture.md#s20)。下表全部为待实施的测试门槛。OpenSpec 格式验证不会执行这些验收，也不能替代 corpus、oracle、fuzz 或行为对照。

| ID | 验收主题 | 首次负责阶段 | 必须保留的证据 |
| --- | --- | --- | --- |
| A01 | 未使用 Methodref 不是调用 | P1 | 同 fixture 的 CP 命中与 X1 consumer 结果对照 |
| A02 | invokevirtual 位置精确 | P1，P0 先验 bytecode spans | 原 entry、方法完整 descriptor、BCI 50、opcode、CP index |
| A03 | annotation/signature/catch 中独有类型 | P1 | 分类别正例、候选过滤与不经过 Class CP 的样本 |
| A04 | LambdaMetafactory 与任意 bootstrap | P1 结构；P3 恢复；P4 增量语义 | implementation handle 和创建/调用的区分；未知最终目标 |
| A05 | nested condy 与共享图 | P1 结构；P4 语义 | visited/cycle、边数/深度预算、共享节点和各 use-site 的 via 路径 |
| A06 | root/11/17 MR 选择 | P1；P4 RuntimeMatrix | Physical 全量与 Java 8/11/17 选择；Manifest 条件及不合规诊断 |
| A07 | WAR 同名类 | P1 物理/选择；P2 解析 | 不同 ordinal/origin、显式 loader/order 与歧义状态 |
| A08 | DEFLATED nested JAR | P1 | 限内物化、超限 Partial、STORED 与 DEFLATED 成本区别 |
| A09 | 历史 jsr/finally | P0 指令；P1 原始 X1；P2 规范化；P3 恢复 | 无需 normalize 的 X1；returnAddress/raw CFG/origin；可靠恢复或 fallback |
| A10 | 缺失 StackMap/debug | P2；P3 无 debug 命名 | 独立 Frame 推导与版本合法性诊断；不得宣称 verifier 通过 |
| A11 | Base.foo 声明、Sub CP owner | P2，P4 深度扩展 | symbolic 原 owner；宽候选/继承扩展后 resolves_to；缺失依赖不能变否定 |
| A12 | accessor/concat 恢复隐藏调用 | P3；P4 现代恢复 | 恢复前后相同 X1 原始边、派生关系与多 origin source map |
| A13 | 成员级失败 | P2/P3 | 同类正常与失败方法并存；representation、quality、execution、诊断分开 |
| A14 | 全范围中断/缺失依赖 | P0 预算；P1 查询；后续各阶段回归 | 已扫描/未扫描范围、终止维度、取消与 resolution coverage，不假 Complete |
| A15 | 冷/热/关缓存完整结果一致 | P5，每个缓存引入时 | 固定 snapshot/query/view、完整运行语义 fingerprint；中断子集不要求相同 |
| A16 | 单方法按需边界 | P2/P3，P0 先验局部 bytecode | Header/Body 物化记录、扩展理由，不读无关 Body、不建全局 XRef |
| A17 | X1 零 CFG/SSA/AST | P1，P2–P5 持续回归 | 构造计数和编译依赖边界；不能仅以“没有输出源码”代替证明 |
| A18 | 分析期间输入变化 | P0/P1，各缓存/并行阶段回归 | 固定字节源或可检测变化中止；跨快照 token/缓存隔离 |

## 语料与发布要求

每个 fixture 记录来源、生成命令/编译器版本、输入摘要、目标 dialect、runtime/output profile 和预期能力。真实历史 javac/ECJ 样本与手工构造边界样本分别标记，现代 `--release 8` 不替代历史 codegen。

输入矩阵覆盖 CLASS/JAR/WAR、Boot executable/deployed、MR、ZIP64、STORED/DEFLATED nested、同名/同字节多 origin、45–52 历史版本和 53–71 的逐 feature 注册。对抗矩阵覆盖截断、未知 CP/opcode、switch/wide、过长 attribute、非法索引、循环和超预算。

输出支持矩阵分别记录 parse、X1、resolution、decompile-quality、output-level。语法检查、重编译和语义验证各自记录；生产引擎不执行输入，只有已知受控 fixtures 参与隔离动态对照。完整执行与预算中断分别比较，冷/热一致性只适用于相同语义配置的完整执行。

当前 P0 的 Foundation 2.1–2.5、容器回归 3.1、classfile 对抗/性质回归 3.2、预算/取消/局部性门禁 3.3，以及 README、五维支持矩阵、低内存 CI、公共示例和实际测试记录 3.4，均已由对应任务和证据证明。公共 Engine 与有界 JSON CLI 已通过真实二进制入口验证；bytecode 边界另由受控 JDK 25 oracle 交叉检查，并以固定 ECJ 4.6.1 生成的 45.3–52.0 历史目标语料验证版本读取和 `jsr`/`ret` 边界。CLASS/JAR/WAR、重复项、小型真实 ZIP64、压缩、CRC、快照、MUTF-8、未知/嵌套 attribute、switch/wide、截断和属性/指令性质矩阵已复核；P0 的十一项请求 limit 均有边界证据，其中 `nested_depth` 是非累加高水位，其余十项维持累计/elapsed 语义（P2 1.3 另加入六个计费维度与第二个高水位 `dependency_depth`，当前 limit 面共十八项，见支持矩阵），公共入口预取消及枚举/读取/CP/handler/instruction 中途取消不伪造 Complete，未请求方法的非法 Body 不影响所选方法，P0 生产模块和依赖中不存在 XRef/CFG/SSA/AST/IR。Linux x86_64 的 stable、MSRV 1.88 和 supply-chain CI 已在 run `35168810088` 及后续完成记录 run `35169222351` 全部通过；Linux aarch64 有本地单作业证据。P0 3.5 的最终门禁、主规格同步与归档已完成，归档记录位于 `changes/archive/2026-09-17-establish-p0-foundation/`，已生效规格位于 `specs/`。P1 1.1 已建立 query relation、canonical consumer schema、PhysicalView/RuntimeView、RuntimeProfile、LoadDomain、显式 standalone/archive class location 与有向 nested origin chain，并以 A06/A07 类型测试证明身份、未知策略和序列化边界。P1 1.2 已用真实 DEFLATED nested fixture 验证 A08：显式 artifact-tree 保留完整 origin、Boot/WAR 物理 evidence、STORED/DEFLATED 重读、`nested_depth`/entry/bytes/cancel Partial coverage 和失败 sibling 隔离；普通 enumerate 仍不递归，也不声称 MR/runtime selection。P1 1.3 以 Manifest evidence 与唯一 winning level 派生标准 MR 选择与合规诊断（A06）。P1 2.1–2.4 实现结构 XRef：共享 class candidate 规则与损坏候选诊断、code/metadata/bootstrap/resource consumer（含 Record component 与 Code 内注解、实际使用点 descriptor 类型）、deferred bootstrap 图与 `UnsupportedAnalysis` 证据保留（A01–A05、A17）。P1 3.1–3.2 实现 target-bound 游标与分页、CLI `query`；3.3 建立验收语料索引、结构 XRef golden、proptest 性质与有界 fuzz 门禁（A14、A18）；3.4 完成文档同步、完整 CI 与归档，主规格新增 `artifact-views`/`query-api`/`structural-xref`。P1 的最终候选 CI run `35233298026`（stable/MSRV/supply-chain/fuzz-smoke 四个 job 全部 success）与逐项证据见 `changes/archive/2026-09-17-p1-query-xref/verification.md`；P2 已交付 1.x/2.x/3.1–3.3，3.4 为未提交候选；2026-09-18 review 确认跨 loader、returnAddress、异常 locals 与预算缺口，先执行 0.x 修正。A11 单 loader 基线仍有证据，跨 loader 需重新验收；A09 尚未完成规范化；A10 尚未实施 Frame/SSA。P3–P5 仍为 planned / not implemented。只有真实测试记录满足对应任务和 change 的出口门槛后才勾选实施任务并归档。
