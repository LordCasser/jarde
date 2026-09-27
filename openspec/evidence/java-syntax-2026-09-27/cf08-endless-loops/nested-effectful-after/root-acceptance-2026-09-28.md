# CF-08 外层 if 的带效果循环：主线独立验收

主线 `452d18f0` 合入代理的 `2ced6d82` 后，root 独立构建 CLI（SHA-256 `d30864f9725f2dc6eff7d77f786b159171fffbb123e315b2472f358e24396819`），重跑冻结 `nested-effectful-baseline/replay.py --require-jarde`。脚本校验固定 JADX revision/算法文件哈希、输入和 Java 8 class SHA；原、JADX、Jarde **完整类源码**各自以 `javac --release 8 -g:none` 重编，`java -Xverify:all` 四组均输出 `-1:0 / 8:1 / 3:0 / 8:1`。Jarde `pick` 无 `@bytecode`，所需 BCI 11/17/26/35/38/44/50 有来源；外层 If 持有内部循环和 BCI 44 尾段，BCI 50 只由后续直线 Region 持有。

root 对最终差异逐项复核：只给直接外层 if 臂传递有界入口，原带效果双出口的 CFG/SSA/调用证明未放松；新边界核对父分支、另一臂、内部 join 两个前驱、唯一直线尾段及父边界的准确正常入边。失败路径在分支快照下撤回已访问集合，未更改 `build.rs` 局部安全门，也未新增 AST/Region 类型。五个 Java 8 负例的 class 由 root 重新编译并与提交的字节码逐字节比较，`java -Xverify:all` 五条均可运行；Rust 测试确认额外入口、异向出口、绕过尾段、第三入边和异常边仍保留引用，预算/取消无半份输出。

主线 `cargo test -p jarde-java --tests --locked` 全通过（lib 233/233），`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-nested-effectful-loop-arm --strict` 通过。root 另重放顶层带效果双出口与纯双出口，三方完整类源码重编和验证运行均一致。固定 `TestNotIndexedLoop` 的原 class 与 JADX 输出正常，Jarde 仍引用、完整源码缺返回，故 CF-08 整单元保持已证差距；其中对象构造、虚调用与内层长度分支需另按证据拆分。外层 null 臂的结构性 goto BCI 6 由 BCI 4 Region 所有，但现有 source map 不给该跳转单独文本段，不纳入本切片七个指定 BCI 的来源门槛。root 专用 Cargo target 在验收后清理。
