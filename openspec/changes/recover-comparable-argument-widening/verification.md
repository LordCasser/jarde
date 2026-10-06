# 验证（change `recover-comparable-argument-widening`）

与 `recover-charsequence-argument-widening`、`recover-enum-argument-widening` **一次实现**（同一表族、
同一判定函数、同一测试入口）；本文件记本片自己的锚与行。

## 实现（`crates/jarde-java/src/build.rs`）

`platform_interface_argument_widens(java_release, presented, required)` 的第三张 `const` 行表
（`java.lang` 九行，落点与 CharSequence 表并列同一函数）；调用点在 `invocation_argument` 的引用扩宽
序列中一处，命中即既有 `cast_argument` 呈现。

## 行表与 javadoc 出处

| 行 | 呈现类型 → 目标 | release 8 事实 |
| --- | --- | --- |
| 1 | `java.lang.String` → `java.lang.Comparable` | header 自声明 |
| 2–7 | `java.lang.Byte`/`Short`/`Integer`/`Long`/`Float`/`Double` → `java.lang.Comparable` | 各自 header `implements java.lang.Comparable<…>` |
| 8–9 | `java.lang.Character`/`java.lang.Boolean` → `java.lang.Comparable` | header 自声明 |

转录（`javap`，rt.jar sha256）见 `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`。
`java.math`/其余 JDK 实现者不入表（spec 决策 1/Open Question 2 的 Non-Goal）。

## 锚实测 vs 预期（两条 javac 腿逐字一致；`essential` + source map 入口）

| 锚 | 预期 | 实测 |
| --- | --- | --- |
| `RG.callGen`（巡查主锚） | 调用点恢复 | `return (java.lang.String) max((java.lang.Comparable) "a", (java.lang.Comparable) "b");`，整类 refusals=0 |
| `RG.callGen2`（**装箱参与**） | 装箱后入表判定 | `return (java.lang.Integer) max((java.lang.Comparable) java.lang.Integer.valueOf(1), (java.lang.Comparable) java.lang.Integer.valueOf(2));` |
| `RG$IntNode.cmp`（形锚，巡查健康面） | 零回退 | `((java.lang.Integer) this.val).compareTo((java.lang.Integer) arg1.val)` 逐字未动 |
| `CO.callGen`/`callGen2`/`same`（冻结形） | 同形 | 与上同形；`same` 证明非字面量实参同样入表 |
| `CO.main` | `cmp` 形 + 两调用点合体 | 文本逐字钉（双腿一致） |
| 负例 `COX.big` | `java.math.BigInteger → Comparable` 仍拒 | 拒绝文本逐字（`presents `java.math.BigInteger` … requires `java.lang.Comparable` …`），拒绝计数恰为 1 |
| spec 负例（`Object → Comparable`） | 仍拒 | 单元级实测（`platform_interface_argument_widening_reaches_exactly_the_table_rows`）；`Object` 实参**无法由 javac 源产生**（源级非法），故不入 fixture |

## 冻结与行为

- `tests/fixtures/recover-comparable-argument-widening/`（`CO`/`COX` 源 + 两腿 class + README 记 sha256）；
- `tests/recover_platform_implementer_argument_widening.rs`：`the_comparable_positions_are_presented`
  （4 文本逐字 + 全类零引注）、`the_out_of_package_comparable_implementer_still_refuses`（1 负例 + 计数 1）
  ——两腿各断言一次；
- ignored replay：`CO`（含 `CO$Node`/`CO$IntNode` 两个伴随单元）剥离后双腿编译运行，答案 `1/b/2/b`
  与 fixture 自身 class 逐字一致（实测通过）。

## 零回退

- 既有 java.util/Throwable 表与数组闭集逐字未动；`RG` 其余成员、`CO$IntNode`/`CO$Node` 的泛型
  Signature 注记与擦除呈现逐字未动；
- 用户类 `Comparable` 的 snapshot 通道未改（本表只答平台行）。

## 移动的既有 pin（如实记录，2 处）

`tests/same_class_generic_binding.rs` 的两个 fixture（`SCGA`/`SCGF`）的 `main` 原本以
“`java.lang.String` 值到擦除 `java.lang.Comparable` 参数、本层无证据 + `void` 体整段响亮拒绝”为
**该测试自己的边界断言**。本表落地后两处都恢复整段：

```text
        SCGA local1 = new SCGA();
        java.lang.String local2 = "b";
        local1.note((java.lang.Comparable) local2);       // SCGA（方法调用族）
        …
        SCGF local1 = new SCGF();
        java.lang.String local2 = "b";
        local1.add((java.lang.Comparable) local2);        // SCGF（字段读族）
```

两条断言据此改为“新呈现存在且不再出现 `jarde_refused_body();`”，注释重写为变动后的原因；测试的其余
断言（`note` 的泛型签名投影 + same-class binding 证明、`kept` 字段的 `field_generic_body_unproved`
拒绝、`reflect_and_run` 的反射/行为双腿一致）逐字保留。**严格更好**：被拒整段 → 恢复整段，运行与反射
仍与原 class 一致（测试自带的 runner 双腿实测通过）。

## 合并派发节（与 charsequence/enum 片共用）

- 若干部件是四表共用的，其证据记在 `recover-charsequence-argument-widening/verification.md`：
  行来源转录（`widening-row-sources/`）、数组位谓词（及它曾误改既有通道的自查修正）、语料指纹与
  fixture 人口计数、全门禁表（308 targets / 3021 passed / 0 failed / 52 ignored，fmt/clippy/openspec 全绿）；
- 本片自己的锚/负例/冻结与 replay 见上。
