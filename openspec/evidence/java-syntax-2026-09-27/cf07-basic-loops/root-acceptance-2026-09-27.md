# CF-07 主线独立验收

root 审阅 `d9b503bd` 的生产差异：只在既有 `Frame::loop_body` 的 scope 内加入经 CFG/SSA 证明的唯一 `iload; ireturn` 叶，循环 walk 仍要求实际覆盖该叶；没有放宽 `build.rs` 的局部作用域、改变 Region/AST，或把 CF-08 的多 break 归入本项。额外入口、共享叶、异常入边、缺值返回和停止预算均由定向负例拒绝。

root 从当前主线重新构建 CLI（SHA-256 `c738c75b0834390de9e8bab46bfc1d7376f0ce90bd9319a0e6804708b3dbb976`），把冻结 [replay.py](replay.py) 独立运行到 `/tmp/jarde-cf07-root-acceptance`。原 class、固定 JADX、当前 Jarde 的完整 `LoopCases.java` 都以 Java 8 编译、`java -Xverify:all` 执行，八行输出同为 `0,10,39,63,9,3,-1,3`。Jarde 源码 SHA-256 与 agent 修后证据一致：`ecbd892b39e3f36d634509de8bec9c4a5b811e96fe5a20033cc38ce896655b23`；源码无 `@bytecode`。

主线定向测试 `p3_loop_terminal_return` 4/4、相邻 `p3_loop_transfers` 5/5 通过，`cargo fmt --all -- --check`、`git diff --check`、`openspec validate own-proved-loop-terminal-return --strict` 通过。全工作区另有独立已记录的测试债务，不能据此宣称全套通过。CF-07 的其余 JADX 循环形态及 CF-06 条件内赋值继续留在 71 项队列；此处验收的是一个有界切片。
