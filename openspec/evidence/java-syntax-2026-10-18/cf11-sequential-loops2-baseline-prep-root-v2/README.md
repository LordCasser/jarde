# Seq2 采集准备 v2（未执行）

仅准备，未运行 JDK/JADX/Jarde CLI/Cargo。归档内 v5 collector 只将已审 v4 的 CLI 路径、SHA及错误文案绑定到当前 aabf 候选，算法、完整上游源码、八case driver和物理parser未变；root已读全部diff并核AST。候选自身CI仍待终态，不冒称下一语法缺陷。

capture.tar.gz：9成员逐字复开，31361B，SHA256 `4acca7ac92e4af1a25f26f0ffc3ba7dd304dfd8f8d2f5f14f902b4b4015c2135`。执行前恢复工具与依赖并重核pins，使用fresh out、target absent；不得将此准备包计为对照通过。

上游断言要求两while、break和两次127阈值判断；源码的lowercases真实初值只有{'a'}。原始default与测试注入configured128语义必须分开，先取真实JVM异常/返回/同对象结果；JADX文字断言不能替代运行oracle。当前尚未对Seq2取首拒或确认需要任何新机制。


root只读Jarde静态复用点（非动态首拒）：region.rs:12485 test_is_pure 经 test_expression_instruction 调用 :12602 snapshot_condition_part，已经证明条件位置的iload;iinc±1及唯一使用等约束。不能先认定Seq2的i--触发StatementFree。:17360起statementful_loop_header body要求单canonical路径；nested if/break若进入该局部消费者会拒，但尚未观察实际调用路径，不据此判定缺陷或新增机制。
