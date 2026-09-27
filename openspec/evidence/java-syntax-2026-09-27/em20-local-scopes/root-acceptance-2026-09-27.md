# EM-20 主线独立验收

Root 将实现 `bf43756bc6528fe21899714a15e5ec605515a60a` 和预算修正 `9107ee1dafaf339ff70d14ac5cf44f28a6807cad` 拣入主线为 `c239ba6f`、`94879dac` 后，重新构建 CLI 并独立执行 [replay.py](replay.py)。原 class、固定 JADX、Jarde 的**完整 `LocalScopes` 类源码**分别通过 `javac --release 8 -g:none`、`java -Xverify:all`，七行 `5,3,0,10,6,2,3` 逐字一致。主线 Jarde 源码与 [`accepted/source/jarde-LocalScopes.java`](accepted/source/jarde-LocalScopes.java) 逐字节相同。

作用域目标测试 12/12、`p3_hoisted_boolean` 固定预算测试 9/9、格式和 OpenSpec strict 均通过。放行限于受证 monitor 清理异常行内 Ref→int 槽位分割及后续循环 φ；其它异常处理器、外层回边、额外消费者继续拒绝。
