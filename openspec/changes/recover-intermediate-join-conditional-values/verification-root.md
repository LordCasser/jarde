# Root 独立验收（2026-09-26）

Region 仅在内层 `If` 的未认领 join 同路径、同 scope、正常边和预期前驱/后继完整闭合时，把后续 Straight 纳入外臂；失败回滚 visited 并引用整段候选。值证明分别绑定内层 Phi、桥接 `Push(int) → iadd → Transfer`、外层 Phi 的两个真实前驱 exit、唯一消费者和 `int` 类型。Builder 先局部构造子条件，在桥接渲染期间临时可见，完成外条件后才发布根 Phi 与折叠分支；拒绝时引用每条候选的物理 BCI。

Root 用重建 CLI 对冻结 Java 8 class 直接生成完整类，未编辑源码。`choose(I)I` 呈现为 `arg0 > 0 ? (arg0 > 1 ? f1() : f2()) + 3 : f3()`，默认与 `all` 正文相同。生成类经 `javac --release 8` 与 `java -Xverify:all`，六条正常/异常输出均与原版和 JADX 逐字一致。报告中内 join 18、桥接 BCI 19/20、外 join 26 有物理来源；外层条件无 fallback。

`cargo test -q -p jarde-java` 全通过（206 个库测试及全部集成测试），其中双 join 定向测试 4/4；`cargo fmt --all -- --check`、`git diff --check` 和 `openspec validate recover-intermediate-join-conditional-values --strict` 通过。verifier 有效的独立调用、抛错除法、外部入口与 handler 负例均保留完整引用，预算及取消停止不发布半成品。严格 Clippy 因 24 条本轮之外的既有告警失败；未在本轮处理。验收后清理隔离 Cargo target。

随后补齐了[负例闭包矩阵](evidence/negative-closure.md)：额外 SSA use、Call/异常/Return 入边、重复前驱、visited 差集与 owner 重叠以直接 proof-unit 检验生产拒绝门槛；未知 `ixor` 运算使用 verifier 有效的 Java 8 class 检验整段引用及物理来源。Region 入边现要求两个不同预期前驱的集合与实际 Normal 入边完全相等，避免重复同源边冒充闭包。`cargo test -q -p jarde-java` 更新后全部通过（210 个库测试及全部集成测试），`rustfmt --check`、`git diff --check` 和 OpenSpec strict validate 通过。任务 1.2、2.2、3.1 据此完成；proof-unit 不声称其注入事实是一个完整有效的 class，组合范围仍限于已证明的桥接形状。

Root 又在隔离 target 独立重跑 `cargo test -q --locked -p jarde-java`：210 个库测试与全部集成测试通过；只检查本项三份 Rust 文件的 rustfmt、OpenSpec strict 和 diff check 也通过。Root 从冻结原 class 独立运行 `GenerateBridgeControls`，生成字节与提交的 `ixor` class 完全相同（SHA-256 `8befd5c4763826d43b7b790983e7a32927993cbefa0657530654c7136bb46fe6`），`java -Xverify:all` 的六条正常/异常输出均可重放。全仓 fmt 受同时进行的成员家族编辑影响，待其稳定后统一复核；本项代码已单独通过格式检查。
