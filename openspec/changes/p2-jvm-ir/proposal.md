## Why

以 P1 完成为进入条件。本 change 规划增加对结构 consumer 之上的 JVM 控制流、栈帧、异常边和符号解析，并建立按需 Demand Resolver 与 Java 8/历史 classfile 的 JVM IR 管线；在无法证明等价时保留可诊断的 bytecode fallback，当前尚未实现。

## What Changes

- 增加平台 Header provider、LoadDomain 下的 Demand Resolver 和 Resolved/Missing/Ambiguous 等结果状态。
- 增加 raw CFG、jsr/ret 子程序分析与有界规范化、Canonical CFG、Frame、stack/local SSA、effects 和 exception region。
- 增加固定 Phase/Pass 契约、origin/diagnostic 传播及 Conservative/Bytecode 输出。
- 分离 `representation`（Java/Bytecode/Mixed）与 `quality`（Structured/Conservative/Fallback），并独立记录 `compile_status`、`semantic_validation` 和 `verification`。
- 保持结构 XRef 可独立运行；P2 不承诺 Java 8 高阶源码恢复。

## Capabilities

### New Capabilities

- `demand-resolver`: 按 RuntimeView、LoadDomain 和平台 providers 按需解析符号与 dispatch 候选。
- `jvm-ir`: 从保真 bytecode 建立 raw/canonical CFG、frames、SSA、effects 和异常模型。
- `conservative-output`: 在 IR 或规范化无法证明时保留 Bytecode/Mixed 表示，以独立 quality 描述 Conservative/Fallback。

### Modified Capabilities

无。P2 规划消费 P1 的 RuntimeView 与 P0 的 classfile/coverage 契约，并补齐 A11 的 Base/Sub 候选扩展与声明解析；反编译不要求运行 X1 或建立全局 XRef。契约需要变更时通过 OpenSpec 显式修订，不作旧接口兼容承诺。

## Impact

影响 `jarde` 的 resolver、IR、frame/SSA、exception/effect 和 output model，以及 `jarde-cli` 的局部方法分析接口。规划复用已评估依赖，不预设新依赖。验收重点为 A09、A10、A11、A13、A14、A16、A17；本 change 仅定义未来实现，尚未实现。
