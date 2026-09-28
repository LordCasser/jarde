## Context

[固定 Test9 证据](../../evidence/java-syntax-2026-09-28/cf16-test9-catch-finally/README.md)记录了 `[2,43)→53 any` 和 `[53,55)→53 any`。BCI 0–1 先把 null 存入 local 1，BCI 2–11 再用资源查找结果覆盖同槽；BCI 42 把字符串保存在 local 3。正常清理是 BCI 43–50 的 `ifnull`/`close`，51–52 返回；异常 handler 从 53 保存 Throwable，在 55–60 对同一个 local 1 做条件关闭，并于 63–65 重抛。自保护行只覆盖 `astore`，不覆盖 `close()`。

固定 JADX 集成测试默认走 DX。JADX CLI Java-input 输出把资源赋值和 finally 检查分成两个变量，资源存在时观察到 `close=0`，因此它是错误反例；原物理 class 和 JADX `--use-dx` 输出为 `close=1`。现有 `Shape::Finally` 的单行无条件副本和 `ConditionalFinally` 的字段型 void 清理均不能直接证明本地可空资源、保存返回值及 handler 自保护行。

## Goals / Non-Goals

**Goals:** 在固定双行布局中证明同一 local 1 的 null 初始化/资源赋值/两份非空关闭、保存返回值和原异常身份；输出完整 Java 8 类的 `try/finally`，覆盖资源存在、缺失及清理抛错路径，全部 BCI 可溯源。

**Non-Goals:** DEX 解析、Scanner 到 try-with-resources 的重写、通用别名推断、Test2/5 或任意编译器的 finally lowering。JADX Java-input 的漏关输出不得作为正确性基准。

## Decisions

1. **在既有 Guard/Region/Build 中建立窄证书。** 按两行的范围、优先级、handler 自保护 store、两个条件清理副本和唯一返回/重抛完成核验；Region/AST 仍用普通 `try/finally`，不新增 pass 或语法节点。可复用既有 saved-return SSA 与 cleanup-copy 判等的证明工具，但不把现有 `Finally` 单行或 `ConditionalFinally` 字段型证书加上一组无意义的可选字段。新物理形态只在证据闭合时主张，其他形态保留安全拒绝。
2. **把资源变量的定义使用链当作契约。** local 1 的 null 初值在 try 外，资源赋值在 try 内；无论赋值前抛错、赋值返回 null，还是已取得资源，都必须由两份清理读取相同 SSA 到达值并对该值判断非空、调用同一 `InputStream.close:()V`。证明 `scanner.hasNext()/next()` 产出的值先保存至 local 3，清理后返回；handler local 4 保存/重载/抛出同一 Throwable。仅相同 `close` 名称或同槽号均不够。
3. **完整边与原子交付。** 穷举受保护块、正常 copy、handler 的 canonical 异常/正常/返回边；无额外入口或出口，清理自身抛错不能重入 handler。Region 接管所有物理块并保留条件分支，Builder 在循环/局部声明、来源和输出预算都可呈现后一次发布；任何失败回滚并保留 fallback。JADX 的 `MarkFinallyVisitor` 可参考副本定位次序，不复用其会误配 Java-input 变量的相似文本判断。
4. **行为基准使用物理 class。** 用原 class、原 Java 8 转写、JADX DX 和 fresh Jarde 完整源码对照资源存在/缺失、正文异常和 close 异常；固定 Java-input 结果在资源存在时的 `close=0` 保留为回归反例，不要求 Jarde 匹配它。每个变异负例均须 `java -Xverify:all` 有效，并与原目标分开记录语义。

## Risks / Trade-offs

- **局部变量拆分导致有效资源漏关** → 同一赋值/读取 SSA、非空分支和运行探针同时验收，资源存在路径必须 `close=1`。
- **自保护范围误盖清理** → 精确核 `[53,55)` 和 handler 自边；扩围至 `close()` 的近邻拒绝。
- **Scanner 构造或读取抛错时的值状态复杂** → 分别覆盖赋值前与赋值后异常，验证 null/非 null 清理次数及 Throwable 身份。
- **窄证书被误认为一般化 TWR** → CF-16 账本只标固定 Test9 切片，其他 profile/资源形态另行取证。
