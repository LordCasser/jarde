# root验收：同类泛型调用消费位

本片已完成，任务9/9。实现提交7172fdcb的实际MSRV失败已在276a1c31修复；[实际实现CI 37889792696](https://github.com/LordCasser/jarde/actions/runs/37889792696)四job全部成功，[root从完整实际日志核验](results/root-input-audit/ci-276a1c31-root-verification-v3.json)两个seed各348targets、3304passed/0failed/93ignored，真正JDK25 oracle、显式Java对照、Clippy和strict OpenSpec均成功。最终交接文档提交的当前HEAD与CI以handoff开头查询命令核对；不把旧快照冒充新源码。

## 最终源码与本地门禁

唯一MSRV收尾改动：report.rs中Not/Local的if-let match guard改为等价内层match。Local字段与锚点原样，其他子表达式仍Other；Rust版本、CI和预算/拒绝边界不变。

最终CLI9 SHA256 `5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006`，八文件真实v42 archive SHA256 `fc4220ac5eb6906a786340a6ef5f8dd840ccbc14790df7bcee115f7c9f78fce8`。CLI8/v41只证明历史快照，CLI9补验不冒用旧二进制。

- [v45 root本地汇总](results/root-input-audit/final-msrv-acceptance-v45.json)。
- [MSRV/308 Java层单测/CLI构建独立核验](results/root-input-audit/msrv-local-root-v45.json)。真实1.88命令为`rustup run 1.88.0 cargo check --workspace --all-targets --locked`；v42本机Homebrew cargo拒绝+toolchain的入口错误保留。report::tests错误筛选0项不计验收，追加308项真实全通过。
- [v41完整本地门禁](results/root-input-audit/final-local-gates-v41.json)：两个seed各348targets、3304passed/0failed/93ignored；显式ignored P3三项、constructor一项、bound receiver一项；strict OpenSpec323/323。fmt/diff、定向101项（另1ignored）、graph2项及CI同口径strict Clippy通过。
- [实际7172 MSRV失败](results/root-input-audit/ci-7172fdcb-msrv-failure.json)，原始日志保留；没有提高MSRV或放宽检查。

## 最终对照

[CLI8/v41完整独立验收](results/root-input-audit/final-candidate-acceptance-v41.json)保持原样：140主矩阵+Nested4，完整重编/行为144/144，完整泛型API96/144（baseline72/144编译、20/144 API）。44控制实际输入、完整source和Probe与CLI6一致，九族可靠拒绝、两族正控制；拒绝不计API恢复。

GC09原口径：字段46输入44完整编译/行为、26完整API，SCGB两既有编译拒绝；构造80行为、36完整API；raw64行为、字段/方法/类API60/52/60，旧已接受API零回退。

[CLI9 root独立输出核验](results/root-input-audit/msrv-output-equivalence/root-verification-v2.json)：324次实际调用（matrix144、Nested4、field28、ctor80、raw68），每项输入SHA、原/新真实exit、source和完整report核对。text source216份逐字节一致，JSON108份及text报告仅usage.elapsed_millis允许差异；RawNewHold四次exit4与CLI8一致。首版TOML parser未处理CLI报告的null而失败的结果保留，v2只对整行null token作无碰撞解析，不改原始输出。

18个source-rebuilt字段临时jar已缺失，未伪造zip字节身份：[完整补验](results/root-input-audit/field18-msrv-final-verification.json)、[root直接解析永久ZIP/class与完整source](results/root-input-audit/field18-msrv-source-equivalence-root.json)。原source SHA、同JDK同参数重建的全部class SHA18/18匹配CLI8记录，完整候选source亦18/18相同。原/baseline/CLI9的显式空classpath/sourcepath完整编译与新classes-only -Xverify行为18/18；候选API输出12/18，baseline10/18。JADX14/18成功，ListWrong/ParamReassigned四个真实编译失败保留。永久输入、命令双流及失败attempt保存于results/candidate/field18-msrv-final。

合计342条实际候选请求全部交代清楚；334个输入row与342条class-source请求是不同分母，没有将可靠拒绝或JADX失败计成功。

## 架构验收与剩余范围

复用Signature擦除、同次opaque AST/Code/SSA/InitRecord及method→field publication。有限callee-first staging、最终完整incoming-use/overload证明和八字段窄source overlay原子提交。独立NoBody、具体无TypeVariable声明及constructor自身method formal保持既有路径；stop/cancel保留此前完整独立commit、未提交组移动回退。普通NotThisRule空record不付构造事实复制费。没有新crate/parser/IR pass/fixpoint、容器或跨类推断。

getter反向字段依赖、容器/wildcard、任意alias/phi、继承/跨类、this/super委派、完整成员类TestGeneric8与SCGB正文仍是后续边界。本片不代表整个generic或全部JADX语法追平。共享Signature解码/缓存、物理事实复制计费属于另片债务。冻结输入应永久保存，旧field临时ZIP缺失教训已在本轮补证，未扩展生产机制。

## 主线与磁盘

14辅助树均detached、干净、提交为main祖先，本地/远端仅main，无分支占用。Codex保护副本保留。两次主仓cargo clean移除约21GiB，当前无target/fuzz target；[清理记录](results/root-input-audit/cargo-clean-final-v45.json)、[工作树审计](results/root-input-audit/worktrees-pre-delivery-v45.json)。全部实现已合入main，任务3.3和handoff已更新。本轮只交接该里程碑，后续语法工作按账本另行开展。

完整历史保存在[此前验收记录](results/verification-history-before-msrv-final.md)，不把旧状态作为当前结论。下一位agent按71单元账本和JADX具体测试/算法确定下一个明确家族。
