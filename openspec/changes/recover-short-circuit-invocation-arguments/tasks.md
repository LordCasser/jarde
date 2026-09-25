## 1. 冻结三方证据与消费者边界

- [x] 1.1 核对 `MixedBooleanArgument` 源码/class SHA，原/JADX/Jarde 完整类 Java 8 重编与八路径 JVM 值/三调用计数；从字段和直接返回任务验收后的当前 CLI 复测 baseline，避免缓存版本混淆。
- [x] 1.2 审计现有 `ShortCircuitValue` 的 region/proof/emitter、Methodref 描述符和 invocation argument 消费路径，冻结单 `(Z)V` 静态调用与多参数/实例/第二消费者/异常边负例。

## 2. 单布尔实参的唯一消费

- [x] 2.1 扩既有图 consumer anchor，仅在真实静态 `(Z)V` 调用位于两个 producer 的唯一正常汇合处时认领；额外入口、异常边及回边仍整体拒绝。
- [x] 2.2 在 SSA 与真实 Methodref 上证明 Phi 唯一 use、参数恰为 `Z`、无接收者或其它参数，保留字段/返回证明；拒绝任何额外效果、参数重排或不明目标。
- [x] 2.3 从现有 `Conditional` 经 invocation argument 适配成一个 `Call` 语句，保留后缀和测试、producer、goto、调用、return 的来源；完整类八条路径与原 class 的值和三计数逐字一致。

## 3. 独立验收

- [x] 3.1 复跑混合字段 16 路径、直接返回八路径、纯两/三测试、额外入口、异常边、双消费者及相邻调用顺序负例，不把可编译但没调用 sink 的输出认作通过。
- [x] 3.2 运行格式、定向测试、适用 Clippy、`openspec validate recover-short-circuit-invocation-arguments --strict`，记录剩余边界和 Cargo target 清理；只勾选有证据的任务。
