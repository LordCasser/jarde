## Why

CF-16 取证队列的 [TestFinally 家族巡查](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)确认四个文件整方法回退。本片取其中与已验收 Test14 条件清理证书最近、正文最简的 `TestFinallyExtract`（Tf4）：正文把局部布尔标志置真、保存返回值，两份清理副本各自以 `iload flag; ifne` 门控同字段的读改写（`result -= 2`）。现有 `prove_conditional_finally` 只证"try 从 BCI 0 开始 + `this` 字段判空 + 同字段 `close()` 调用"一种文法，Tf4 的前置标志初始化、局部条件与字段更新清理均超界；主线对固定 Tf4 类 `test()` 整方法回退（`jre_region_exception_edge`@0 + 5 未覆盖块）。JADX Java-input 结构可恢复（其 `z`/`z2` 重命名疑语义失真，仅作参照，原 class 是行为基准）。

## What Changes

- 在现有 FINALLY Guard/Region/Builder 通道内新增"局部标志条件清理"有界证书：两行 any 表（正文行 + 自保护绑定行）、两指令 lead（`iconst_0; istore slot`）、受保护正文恰一次置真同 slot、保存返回与 pending Throwable 身份、两份逐指令同形的"局部条件 + 同字段读改写"副本（`getfield F; iconst K; isub; putfield F`，F/K/op 两份一致）；输出唯一 `try/finally`，清理体经既有字段复合更新拼写为 `if (!flag) { result -= 2; }` 一份。
- lead 复用 `Plan::lead` 既有通道；不新增 Shape 之外的实体、公开 IR 节点、CLI 开关或依赖。Test14 字段判空证书与其它两行形状证书不放宽。
- 以固定 Tf4 类与同布局探针变体做原/固定 JADX/Jarde 三方 Java 8 重编、正常与异常路径 `java -Xverify:all` 行为对照、全部物理 BCI 来源与 verifier 有效负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的正文置真局部布尔标志门控两份同形字段更新清理，可恢复为唯一 `try/finally` 的 `if (!flag)` 清理。

## Impact

仅 `crates/jarde-java` 私有 Guard（新 prove 函数与副本文法）、Region（既有 finally 正文通道纳入 lead 后正文）与 Builder（既有字段复合更新呈现）及其测试；Tf1/Tf2/Tf3（可空局部、构造返回、提前返回变体）不在本片内，随后按证书邻近度另片。与本批已合入的 `recover-void-loop-finally`、`recover-preceded-statement-catches` 无共享函数；基线为主线上两者之后。
