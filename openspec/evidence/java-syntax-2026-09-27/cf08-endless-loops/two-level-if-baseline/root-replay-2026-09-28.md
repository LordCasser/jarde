# CF-08 二层 if 红基线：主线独立复放

主线 `47800cfc` 在独立 Cargo target 构建 CLI（SHA-256 `e3a8a74f1b5a303efe8aadd84c00616918c161a5d1934352e1a7d0701a6c0fdb`），root 重新运行 `two-level-if-baseline/replay.py`。脚本核输入和 class SHA、固定 JADX revision 及七个算法/测试文件哈希；原 class 与固定 JADX **完整 Java 8 类源码**各自编译、`java -Xverify:all` 四行同为 `-1:0 / -1:0 / 3:0 / 8:1`。Jarde Region 在外层 BCI 0 报 `jre_region_arms_do_not_meet`，循环块 `[23,28,37,46,49]` 未覆盖，完整源码因 `local 1 crosses a quoted fallback region` 缺少返回而不能编译。此为安全拒绝，不是语义等价。下一片仅处理二层条件下的三来源 join；`File` 构造和虚调用继续独立保留。
