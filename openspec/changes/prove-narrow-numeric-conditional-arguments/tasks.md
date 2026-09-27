## 1. 在现有条件值和调用路径内证明窄数值类型

- [x] 1.1 让字段型一臂与范围内 int 常量一臂在已证条件值闭包内获得正确 byte/short Java 类型；以 `byteField`、`shortField` 正例及错 descriptor、超范围、额外消费负例验证发表前原子性。
- [x] 1.2 将现有 byte 条件实参逐臂判定收敛为 B/S 的最小共用路径，保持 `(B)I`/`(S)I` 的物理目标；以 `castByte`、`castShort`、`shortConstant` 的 true/false 运行结果和原 overload descriptor 验证。
- [x] 1.3 证明新增窄化节点的物理来源和预算/取消停止，确认非恒定 int、异常边或未证目标签名仍引用而不发布部分方法；运行对应来源和预算目标测试。

## 2. 固定三方与回归验收

- [x] 2.1 将 [CF-05 脚本](../../evidence/java-syntax-2026-09-27/cf05-numeric-condition/replay.py) 提高为修后门槛，确认原 class、固定 JADX、Jarde 的完整 `ConversionCases` Java 8 源码均重编、`java -Xverify:all` 运行 20 行一致，完整 `ConversionBasic` 十行继续一致。
- [x] 2.2 运行相关条件值、窄化/重载、预算和来源测试，`cargo fmt --check`、`cargo check --workspace` 及 `openspec validate prove-narrow-numeric-conditional-arguments --strict`；记录未覆盖的 Smali 泛化转换并清理 Cargo 产物。
