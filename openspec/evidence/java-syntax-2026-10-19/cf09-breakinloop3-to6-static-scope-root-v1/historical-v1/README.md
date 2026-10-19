# TestBreakInLoop3–6：源码固定计划

冻结日期：2026-10-19。本目录保存固定 JADX 测试源码及其关联 Smali fixture 的逐字副本，`sha256.txt` 为副本摘要。盘点只做静态控制流与有限输入设计，没有运行 JADX、Jarde、编译器、测试或目标字节码；源码推导出的调用次数不视作实测。

## 固定原件

- `TestBreakInLoop3.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop3.java`
- `TestBreakInLoop4.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop4.java`
- `TestBreakInLoop5.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop5.java`
- `TestBreakInLoop6.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop6.java`
- `TestBreakInLoop6.smali` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/smali/loops/TestBreakInLoop6.smali`

## 范围对比与有限 case

| 固定用例 | 控制流形状与静态可达性 | 最小有限 case 计划 | 观察项 / 验收边界 |
|---|---|---|---|
| 3 | `while (true)` 中 `try` 创建空路径输出流；首个异常置 `tryMkdir=false` 后 `continue`；后续异常根据 `File.exists()` 分支，落到 `break` 后循环外。`check()` 对空路径调用 `writeMore("")`，触发异常路径；首异常 continue 与二次异常退出均可触发。 | 单次 `check()`，沿用空路径，不另造目录或文件。可观察断言由原件 `hasToString("12135678")` 提供。 | 返回为 void；预期不抛异常，异常被内部 catch 消化。stdout 无；`builder` 是内部状态，原件 `check()` 断言其内容。测试入口 `test43` 带 `@NotYetImplemented` 且注释掉 `@Test`，所以原项目常规测试是否执行该动态检查不能由源码确定。环境若令空路径或文件系统语义异常，应单独记录。 |
| 4 | 有限 `for` 中外层条件 `c != '.'` 包住内层 `a || b` 更新后 `break`；`.` 会自然结束循环。方法不抛异常。测试显式要求反编译结果保留一个 `for` 且不出现 `while`，含 debug 与 no-debug 两种反编译输入。 | 原件四值已覆盖：`.`, `a`, `b`, `c`。如只保留最小分支集，可用 `.`, `a`, `c` 分别覆盖自然退出、内层命中后 break、内层未命中仍 break；`b` 是 `a || b` 另一分支的独立输入。 | 返回分别 `1.0`, `1024.0`, `1024.0`, `1.0`；stdout 无；无输入状态变更。动态 `check()` 是辅助方法，两个 `@Test` 实际只验证反编译结构，不调用 check。 |
| 5 | `try/catch` 包住扫描与解析；有限 `for` 接受数字和点，遇到首个其他字符时可更新 multiplier 并 `break`；之后解析累计前缀并向上取整。空串会在解析处进入 catch。`m/M/g/G` 分支与无单位分支可达。 | 原件五值覆盖：`"1.2"`、`"12am"`、`"13m"`、`"1G4"`、`""`。分别覆盖自然扫描结束、小写无效字母后 break 且 multiplier 不变、m 单位、G 单位且后缀不再扫描、解析异常。 | 返回 `2`, `12`, `13*1024`, `1024*1024`, `-1`；stdout 无；输入字符串不可变，无外部状态变更。原件两个 `@Test` 只检查可反编译代码，不调用 `check()`。不要把 `StringBuffer`/`Double.valueOf` 调用数当观测结果。 |
| 6 | Smali 输入包含外层数组循环；条件通过时进入内层数组循环；内层对象若与当前对象同一引用则 `goto` 外层尾部，若 `f` 返回等于当前 long 则继续内层，否则抛 `IllegalStateException`。源码注释明确针对已有 break edge 不应错误插入 continue，造成 continue 丢失/反编译失败。`disableCompilation()` 表明只验反编译源码。 | 由 `test` 使用固定 Smali fixture 做静态验收即可。若要运行行为探针，需另有满足 `test.Test` 接口的 stub，并分别构造：外层条件不满足/满足、内层空数组、相同对象引用跳出、`f` 返回原值继续、`f` 返回异值抛异常；该 fixture 本身没有定义这些接口或输入数据，不能声称已动态触发。 | 静态检查要求恰有一个 `break` 文本命中。动态语义计划：正常路径返回 long；失败路径 `IllegalStateException`；stdout 无；输入数组/元素由探针 stub 管理，方法仅读数组并调用接口，不从源码看到数组写入。stub 的可达路径及调用次数须由实际执行观测，不由源码推断。 |

## 批次与机制判断

3、4、5 都是 Java fixture，可作为同一轮固定库存的有限验收候选；但不要默认共用一个变换机制：3 的关键路径是 catch 后 continue、第二次异常退出以及循环后 break；4 是简单有界 loop 中嵌套条件后的 break；5 将 break、循环后解析和 catch 汇合到方法返回，且 multiplier 跨出口存活。4 与既有 Loop2 的单 protected 区域/单 handler RetryLoopFacts 只能作为待 baseline 比较的假说，结构证据不足以判定同族。3 也有异常 handler 与回跳，但其首错重试及第二次错误的不同出口，应单独留意。

6 是 Smali 专用 fixture、关闭编译，目标是保留已有 break edge；它需要独立静态验收批次。它的嵌套数组循环、对象身份跳转、接口调用失败边与 3–5 的 Java 源码形状不同，不应与前面几例共用验收机制。

## 账本边界

本文件仅记录固定源码形状与有限 case 计划，不据源码宣布 Jarde 缺口，不建立 OpenSpec，不创建 collector，不更改产品、fixture 或账本。若后续审计发现架构债务，应另行拆分记录，不混入当前范围。
