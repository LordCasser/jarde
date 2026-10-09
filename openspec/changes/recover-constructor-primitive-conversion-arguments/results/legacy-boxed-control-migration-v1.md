# 旧 direct `boxedDirect` 控制迁移

## 结论

本迁移只把 `tests/p3_heterogeneous_array_initializers.rs` 中的 `boxedDirect` 从构造拒绝控制改成正例。它保留在完整 direct 类报告里其它未恢复方法的检查，也保留 `numberGridDirect`、`collectionGridDirect`、`ownGridDirect` 的数组组合拒绝；因此本测试不宣称完整 direct class-source 成功。

旧负例与当前支持范围不符：`boxedDirect` 的六个 wrapper 构造实参均是 `mark(n)` 的返回值，依次执行 `i2b`、`i2s`、无转换、`i2l`、`i2f`、`i2d` 后传给相应构造器。它们属于已放行的 `PrimitiveConversion` 参数链，转换没有额外副作用，调用顺序由六个构造实参表达式保留。原测试把五类转换误标为 interleaved-effect 拒绝、Integer 误标为 shape 拒绝；root 的 `root-workspace-seed1-v2` 验证实际因 Byte 已 `presented: true` 在旧断言处失败。

## 冻结证据

- 规范输入源码：`openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/Main.java:68–74`。方法按顺序创建 Byte、Short、Integer、Long、Float、Double，并分别调用 `mark(1)` 到 `mark(6)`。
- javac8/javac23 的原始 `javap -c` 输出分别为 `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/javac8/logs/direct_javac8_javap-main.stdout`（SHA256 `ae496a7fa34352d5cd68e49ce81e2e801b657752a76222785010b497aea24424`）与 `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/javac23/logs/direct_javac23_javap-main.stdout`（SHA256 `26fe4e5c64e66942e93fe9da460f9b1de44be63fd5221c6b2639e4666dbe11e8`）。两腿方法 BCIs 相同：

  | wrapper | `new` | `dup` | primitive conversion / argument | `<init>` | `aastore` |
  | --- | ---: | ---: | ---: | ---: | ---: |
  | Byte | 7 | 10 | `i2b` 15 | 16 | 19 |
  | Short | 22 | 25 | `i2s` 30 | 31 | 34 |
  | Integer | 37 | 40 | `mark(3)` 42 | 45 | 48 |
  | Long | 51 | 54 | `i2l` 59 | 60 | 63 |
  | Float | 66 | 69 | `i2f` 74 | 75 | 78 |
  | Double | 81 | 84 | `i2d` 90 | 91 | 94 |

- 冻结 legacy CLI 报告 `results/legacy-regressions-root-v1/manifest.json`（SHA256 `dc8a09f2b9d34bd50f2e8b43f5fbcfda23e2f7bfdca58c9970cf81be962528eb`）在 javac8 与 javac23 两腿均记录 `boxedDirect` 为 `structured/java`、无正文拒绝 marker；六条 `NewRecord` 顺序、class、head/dup/argument/constructor BCI 均与上表一致，全部 `presented: true`、`refusal: null`。两腿 SourceMap 都包括表中所有 `new`、`dup`、参数/转换、构造器和 `aastore` BCI。
- 本片 `results/candidate-root-v1/manifest.json`（SHA256 `1b6c1f6f44920d7e849d26576687bfefc76409c66599ca0daa592c5a733cfa71`）两腿完整源集都编译、运行并匹配原始流/exit；其目标方法数据覆盖新转换参数能力。它是相关实现验收，不替代上述 direct 方法自身的 BCIs/record 证据。
- root 已保存的 `results/root-workspace-seed1-v2/stdout` 记录 135 targets / 1328 passed / 1 failed；唯一失败是该测试旧断言发现 `javac8/boxedDirect` 的 Byte record 已成功呈现。这次迁移只更新该过期预期，不改输入 fixture 或产品代码。

## 测试调整

`boxedDirect` 现在加入 `DIRECT_SITE_PRESENTATIONS`，逐条验证六个 class、`new`/`dup`/argument/constructor/store BCI、`presented` 与无 refusal，并要求对应 SourceMap 均有位置。单独断言各个转换写法和 wrapper constructor 在正文各出现一次，同时 `mark(` 总数为六，固定参数求值及副作用调用次数。旧 `DIRECT_SITE_REFUSALS` 中的 boxedDirect 项已移除；`ownGridDirect` 的结构拒绝、两个 grid 的完整 fallback 控制保持原样。

同类旧拒绝断言只读检索结果：`tests/p3_ordinary_new_invokes.rs` 的 `VoidBetween` 仍是独立 `void` 调用夹在 `new` 与 constructor 之间；`tests/recover_statement_position_news.rs` 的相同 refusal 与 field/invocation 参数拒绝覆盖的也不是纯 primitive conversion。`tests/recover_boxed_number_widening.rs` 检查的是方法调用参数的 reference conversion 与边界类型。这些证据不足以判定它们也误拒本片 PrimitiveConversion，故本迁移未改动它们。

此次未运行 Cargo、Git、rustfmt 或 Java；root 将进行审读和后续执行。
