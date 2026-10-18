# 首轮 Seq2 矩阵工具失败

此真实采集终态 failed/exit1。独立 driver javac8 编译成功，随后完整物理 parser 以严格 UTF8 读取 driver 的 Java MUTF8 surrogate 常量失败；1命令/2 raw，零生成源码编译、零 runtime。失败 receipt、collector 和原始流保留，不转记成功。

后续 collector 只把独立 driver 改为现有 compare.parse_class 摘要与字节 pin；目标产物仍完整双 parser。实际新矩阵和独立验收见[成功收据](../cf11-sequential-loops2-generated-matrix-root-v2/README.md)。归档18成员，SHA256 `56fa2c8af6679fc8c30ba28a31d5c9e5fb11d7320b65ed3cc2735d8bd3ce9591`。
