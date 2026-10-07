# 2.1 / 2.2：负向 fixture 与拒绝闭包边界（closing-2x）

日期 2026-10-08。新增 fixture：`tests/fixtures/preserve-local-scope-refusals/`（README 记录源码、
两腿命令、SHA-256 与派生控制）；测试：`tests/preserve_local_scope_refusals.rs` 的
`the_refused_slices_keep_the_instructions_their_closure_covers` 与
`the_boundary_keeps_the_independent_members_and_quotes_only_the_dependent_slice`。

## 1. 2.1 的三族负向 fixture

`ScopeRefusals`（两腿：javac 23.0.1 `--release 8 -g:none` 与真 javac 8 `-g:none`，均无 LVT/行号表）：

| 成员 | 族 | 拒绝句（逐字） | quote | fallback codes |
| --- | --- | --- | --- | --- |
| `savedAcrossFinally(I)I` | 保护区 fallback | `local 1 crosses a quoted fallback region; …` | `0 17 22` | `jre_region_exception_edge`, `jre_region_uncovered_blocks` |
| `handlerComputed(I)I` | 保护区 fallback（handler 计算汇合） | 同上 | `0 8 16 20 22 23` | `jre_region_uncovered_blocks` |
| `nestedHandler(I)I` | 异常边不完整 | `local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write` | `0 10 13 16 20` | — |
| `sharedHandler(Ljava/lang/Runnable;)Ljava/lang/Object;`（`ScopeRefusalsEscape`，未变异） | 嵌套/共享 handler（多 catch 共享一个 handler entry） | 异常边不完整句 | `0 11 14` | — |
| `sharedHandler`（`escaped/` 派生控制，`4d 2c 4c 2b b0` → `4c 2b 4c 2b b0`） | 嵌套/共享 handler 的 catch 参数逃逸 | `local 1 escapes catch parameter scope at region [0, 1]; its catch header cannot declare a method-visible local` | `0 11 14` | — |

两腿在全部五个成员上给出**相同**的句子、quote、region 记录与 source-map origin（测试断言逐项相等）。

## 2. 拒绝闭包的覆盖断言（逐成员）

测试把“quote 覆盖相关定义、handler、汇合/transfer 与区域外 consumer”落成三条可证伪断言：

1. **quote == 切片块集**：quote 行的 BCI 集合恰为被拒切片的块（逐成员钉住）。
2. **切片指令落在这 blocks 内**：每个切片指令（定义/handler/join/transfer/consumer）落在某个被引
   block 的半开区间内；区间来自 fixture 自己的 `javap`（README 记录），逐成员钉在测试的 `blocks` 表里。
   例：`savedAcrossFinally` 的定义 `istore_1@1` 与正常副本读 `iload_1@12` 在块 `[0,16)`，
   handler 副本读 `iload_1@18` 在块 `[17,21)`，区域外 consumer `iload_0@22` 在块 `[22,23)`。
   `handlerComputed` 的**汇合 store `istore_1@22` 本身就在 uncovered 块里**，consumer `iload_1@23`
   在块 `[23,24)`；`nestedHandler` 的两处 handler entry（10/16）、transfer `goto@13` 与 consumer
   `iload_1@20` 全在 quote 内。
3. **origin == 同一块集**：被拒文本的 source-map origin（primary + derived）恰为这些块，读者可按 BCI 定位。
4. **正文无越界名**：被拒成员除 envelope 注释与花括号外没有语句行，绝不出现“catch 内声明、外层读取”的
   越界读取；逃逸控制（catch 参数被写进局部槽并在子句后读取）逐字保留其自身句子。

## 3. 2.2 的边界断言

* **类级独立 sibling**：`siblingKept(I)I` 与任何被拒局部无依赖，整段呈现（`ContainsStatements`、
  `fallbacks` 空、无 `@bytecode`），提升声明在 `try` 之上覆盖两处写入与汇合读；被拒成员的句子与
  sibling 的语句同时出现在类文本里——拒绝不泄漏到无依赖的成员。
* **成员级依赖切片**：`quotedSliceKept(I)I` 写出正常路径的全部语句（`int local1;`…`return local1;`），
  只把 walk 无法认领的 handler 副本块引注为 `// @bytecode 19 20 21 22 23 24 25`
  （`jre_region_uncovered_blocks`，`1 live block(s) … [19]`）；呈现语句按自身 origin 定位
  （`local1 = local1 + local2;` → BCI 11，`local2 = local2 - 1;` → 15，引注段 primary 19）——
  “与局部无依赖的结构保留、依赖切片整块拒绝、mixed/fallback 诊断与 source-map 可定位”四条都在一条
  断言里。
* 被拒成员自身：`ExplanationOnly` + `Mixed`，quote 与 region 记录一致（见 2.1），即依赖切片完整拒绝。

## 4. 证据落点

* fixture：`tests/fixtures/preserve-local-scope-refusals/`（`README.md` 记录五个 class 的字节数/SHA-256；
  `patch-escape.py` 自测 needle 唯一并保 BCI）。
* 派生控制 `escaped/ScopeRefusalsEscape.class`（SHA-256 `b0dcf6b14d81f1d83200adb912bc97b0e857b8c233644bac2759e20dff01fe7a`）
  在 `-Xverify:all` 下由驱动执行：与未变异类同答（`read=java.lang.IllegalArgumentException: bad` /
  `null threw=java.lang.NullPointerException`），但捕获值与方法局部共槽，无法拼成一个 Java 词法绑定。
* 测试：`cargo test --test preserve_local_scope_refusals --all-features --locked`（4 passed / 1 ignored）。
