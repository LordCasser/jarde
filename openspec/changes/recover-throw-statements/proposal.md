## Why

持续审计的十种throw方法中，裸athrow没有语句呈现；简单catch虽然已恢复区域，却因throw缺失导致整类无法编译。字节码事实和终止控制流已经存在，应补齐异常表达式的消费与语句表达，而不扩建异常分析机制。

## What Changes

- 在已接受的区域中恢复`throw null`、参数、new、调用结果和显式cast异常表达式，保持对象身份、求值顺序、次数与异常先后。
- 保留throw及其生产者来源；无法呈现时完整引用相关字节码，不因延迟生产者而丢失效果。
- 沿用既有guard的synthetic rethrow所有权；清栈时丢弃的较低栈值不被误当成throw读取，已经执行的效果仍然存在。
- 增加自写Java8语料及源码/原class/jadx/jarde执行对照；验证实际恢复正文，而不是手写替代实现。

前提是既有SSA、普通引用cast与调用参数类型规则。非目标为finally/synchronized新区域证明、Throwable类层级resolver、checked-exception验证器、通用stack调度和任意无效class修复。此次规格不表示这些能力已实现。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增普通throw语句的消费、效果、来源与拒绝契约。

## Impact

主要涉及jarde-java的现有AST、builder、new消费规则与emitter，以及相应遍历和回归；不改变crate方向、CLI协议或验证结论，不新增库、pass、索引和缓存。与数值比较共用生产文件，实施窗口须串行移交。其它异常区域债务继续单列。
