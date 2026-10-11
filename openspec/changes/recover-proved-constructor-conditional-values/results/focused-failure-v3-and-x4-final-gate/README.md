# 真实完整库首轮失败与最终呈现反例

当前 v3 Java lib 实跑359/1/0；唯一旧 crossBlock unit 仍断言 inner Sites 不能成为候选，已与新增 one-hop path 的语义变化冲突。源码为 new Tag("t", flag ? new Val("a") : new Val("b"))，outer Tag site 保持拒绝。尚不据此改unit就宣称真实语义安全。

Root使用 frozen v2 CLI 实际 all-evidence X4 完整 render：CLI命令exit0、guard正常；root 后处理误用 name.display 导致script exit1，raw/receipt/failure原样保留。独立acceptor读取准确raw identity，actual exit0接受最终结果：heads0/10/22全presented=false，两个inner有准确constructor/arguments且jre_new_conditional_not_emitted，summary3 read/0present/3refused。完整Builder fallback，没有把候选误报成实际New。此诊断只属于 frozen v2 CLI，不是当前 v3 source 的最终验收；新增永久public API回归将在新source上验证相同判定。

v3 finally cargo clean 实际成功，target absent；未执行后续reader/API/updater/CLI命令。
