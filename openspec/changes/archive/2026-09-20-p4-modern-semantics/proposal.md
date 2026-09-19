## Why

以 P0–P3 完成为进入条件。本 change 规划在既有 Registry 和恢复契约上补齐现代语义与深度查询，同时把结构支持和完整 Java 27 源码恢复明确分开；当前尚未实现。

## What Changes

- 增加 module/nestmate/condy/modern concat/record/sealed 的结构、解析和适用版本规则。
- 增加 RuntimeMatrix、X2 dispatch、X3 有界 reflection/service patterns 与 framework query plugin 边界。
- 增加 output level 冲突、现代到 Java 8 降级限制和版本/preview diagnostics。
- 保持不执行 bootstrap、动态加载器或框架代码；未知运行时目标继续返回 Unknown/OpenWorld。

## Capabilities

### New Capabilities

- `modern-classfile-semantics`: 现代 classfile 结构、版本 registry、module/nest/condy/record/sealed 和 concat 语义。
- `runtime-dispatch-queries`: RuntimeMatrix、X2 resolution/dispatch 和 X3 有界动态模式查询。
- `semantic-query-plugins`: 带规则版本、evidence 和 coverage 的框架/资源查询扩展边界。

### Modified Capabilities

无。P4 向 P1 query、P2 resolver 和 P3 recovery 增加现代实现；不重写其原始 facts 或把高版本能力回填成已完成的 Java 8 承诺。

## Impact

影响 `jarde` 的 version registry、runtime matrix、resolver、modern recovery 和 plugin API，以及 CLI 的输出级别与 query 选项。优先复用既有依赖，不预设新依赖；每个现代特性需独立 fixture 和 capability 记录。验收覆盖 A04–A07、A12，以及 53–71/preview 支持矩阵；本 change 当前尚未实现。
