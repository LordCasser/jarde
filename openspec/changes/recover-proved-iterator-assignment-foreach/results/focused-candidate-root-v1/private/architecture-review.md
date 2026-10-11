# 私有候选静态审查 v3

- production `build.rs` 与 v2 SHA 完全相同；测试文件只在现有 `p3_hoisted_iterator_foreach.rs` 中追加四方法 controls 的公开 API 回归。旧固定 CF11 双 foreach断言、45 BCI/source-map核验，以及动态 AnalysisSteps late-stop/pre-cancel 测试均保留。
- 新回归用 `Engine::open`、Java 8 `ClassSourceRequest` 与 default/all/no-rule-details 三种请求。它逐个比较四个目标方法的正文、source map、method identity，并对每条真实物理指令要求 all 请求的 map 有覆盖且至少一个 origin 同时匹配 BCI 和该方法 identity；`assert_utf8_map` 核验 map 边界。
- positive 期望一个 Object foreach；保留 `java.lang.String target;`、`target = \"seed\";`，且这两条都在 foreach header 前；保留循环内的 String cast 赋值与 `Runner.assigned(target)`，随后 `return target;`。三个 controls 只要求仍输出 while 且没有 foreach。测试注释明确这些仅是公开输出控制，不将拒绝归因于某个内部 SSA 门。
- `include_bytes!` 指向后续仓库路径 `tests/fixtures/p3-iterator-assignment/CF11OuterAssignmentTarget.class`。v3 不复制 fixture；按 root 指示由 root 决定后续放置。其 canonical javac8 输入 SHA 为 `28b92d58ce18fb08362bd39b0558de98c3c41830cc8cbc6bd60ec3a183b21956`。
- positive 预期改写尚未对本候选运行；source name、header、cast 输出与所有 API/来源断言均未验收。没有运行测试、fmt、build、CLI、JDK 或 Git。
