## Why

[TestFinally 家族巡查](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)中 Tf1（JADX `TestFinally`）的 `test(Context,Object)` 主线整方法回退（`jre_region_exception_edge`@0 + 5 未覆盖块）。形态：lead 两指令把局部槽置 null，受保护正文把**同一槽**赋为 `context.query(...)` 的结果，两份清理副本各以 `aload s; ifnull exit; aload s; close()V` 门控关闭，返回值保存、原 Throwable 重抛。现有证书都不覆盖：Test14 `prove_conditional_finally` 只证**字段**判空 + 同字段 close（且要求 try 从 BCI 0 开始）；Tf4 证书（`recover-flag-conditional-finally`，实施中）证局部**布尔标志**门控字段读改写。本片是同族的第三变体：**局部可空引用**条件清理。固定 JADX Java-input 结构可恢复（带死代码 artifact，仅作参照）；原 class 是行为基准。

## What Changes

- 在现有 FINALLY Guard/Region/Builder 通道新增"局部可空条件清理"有界证书：两行 any 表（正文行 + 自保护绑定行）、lead `[aconst_null, astore s]`、正文内同槽赋值、两份四指令 `aload s; ifnull exit; aload s; invokevirtual close ()V` 副本（槽/目标一致，仅 exit 不同）、保存返回与 pending Throwable 身份；输出唯一 `try/finally`，清理折叠为一份 `if (cursor != null) { cursor.close(); }`。
- lead 复用 `Plan::lead`；清理体的 null 判与调用复用既有呈现（同 Test14 的 `if` + 调用，接收者为局部声明）；不新增 Shape 外实体、公开 IR 节点、CLI 开关或依赖。
- 与 `recover-flag-conditional-finally`（Tf4）共享 dispatch 注册点但文法独立：本片不依赖也不放宽 Tf4 证书；Test14/Test9 等既有证书零放宽。以固定 Tf1 类与探针变体做三方 Java 8 重编、正常/异常路径 `java -Xverify:all` 行为对照、物理 BCI 来源与 verifier 有效负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的局部可空引用（lead 置 null、正文同槽赋值）门控两份同形调用清理，可恢复为唯一 `try/finally` 的 `if (local != null)` 清理。

## Impact

仅 `crates/jarde-java` 私有 Guard/Region/Builder 及测试；Tf3（正文条件流 + 提前返回 null 变体）不在本片，随后另片。实施顺序：待 `recover-preceded-statement-catches` 负边界修复与 `recover-flag-conditional-finally` 落地后串行开工（共享 `guarded()` dispatch 与 region 装配点）。
