## Why

[单接口子巡查](../../evidence/java-syntax-2026-10-03/single-interface-fold-patrol/README.md)确认窄通道遗漏：单直接静态**接口**子（Y1$StrFn，0x0608）落回旧窄通道 `Candidate`——该通道无接口准入，no-capture 证书判据不匹配接口形态（接口无构造捕获语义）→ 不折叠。类形态（M2$Solo）与多子族（含接口）均折叠；单接口子是 fold/mixed 两片选择交互留下的洞。

## What Changes

- 单静态行改道 `StaticMembers` 折叠通道（行判据同一份，接口 0x0608 准入已在），删除窄通道对静态行的选择（或按取证最小面：窄通道加接口准入——两案以 corpus diff 等价性择一）；非静态候选窄路径不动。
- Y1 家族 jar 输出 `interface StrFn` 嵌套呈现、域内 `StrFn` 源码拼写、重编行为一致（`hi!`/`45`/`[b, aa]`/`8`）；M2/M1/FV 全部既有家族 diff 逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：单直接静态接口子按嵌套声明折叠呈现。

## Impact

`src/member_inner.rs`（扫描分派）及测试；复用既有折叠通道。既有家族/投影零回退。
