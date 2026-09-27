# `TestTryCatchFinally12.test3` 三副本清理独立验收

root 审阅并 cherry-pick 实现为主线 `7e0d8f07`。新增证明只接受三份同一当前实例 `StringBuilder` 字段读取、相同字符串常量、相同 `append(String)` 虚调用及紧随的 `pop`；每份按物理 BCI 和 SSA 栈值证明唯一消费。原布尔字段写入证书被提为独立私有 helper，其行、边、join 证明保持原状。三条固定异常行、两次正常 `goto`、BCI 55 独立 continuation 和原异常重抛仍由既有 `SharedFinallyCompletion::Joined` 证书闭合；无新 Region、AST 或公共 pass。

root 在实现工作树以独立 Cargo target 重跑 `p3_shared_join_finally`（7/7）及 guard 定向单测（1/1），随后独立执行 `replay-test3.sh`。原/JADX/Jarde 最小完整类的 `test3` 三路径 stdout 逐行相同：`call-finally`、`call-npe-catch-finally`、`call-iae-finally`；`java -Xverify:all` 通过。固定方法 0–55 的每条物理指令有来源，四个 canonical block 各唯一 owner，join 55 不在 guard owned 集合。三份参数同时改为 `-catch` 仍恢复唯一 finally；参数、字段、调用目标、结果消费或 catch-all 范围仅改一处的五个 verifier 有效反例均保留引用，不能发布共享 finally。旧布尔固定类和三个既有边界负例的定向测试继续通过。

代理的完整 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、fmt、OpenSpec strict 和 diff 检查通过；root 另核 fmt、OpenSpec strict 和 diff 检查。代理与 root 的专用 Cargo target 分别清理 5.8 GiB、2.6 GiB。全类九路径 Jarde 仍在 `test1` 第一项失败，故任务 1.2 的 `test1/2` 负例、任务 3 和最终 4.1–4.3 尚未完成；不得把整份固定类或 CF-16 单元标为追平。
