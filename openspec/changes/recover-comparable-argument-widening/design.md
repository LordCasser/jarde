# Design：Comparable 实参扩宽的第四张平台小表（CharSequence 姊妹）

## Context（root 已实测）

- 拒绝点与 CharSequence 片逐字同因（DIRECT_EDGES 只覆盖 java.util；snapshot 通道对平台类型结构性不可用——JDK 类层次不在快照内）。
- 主锚：`max("a","b")`（String→Comparable）与 `max(1,2)`（int 装箱 Integer→Comparable）——**装箱参与**（iconst → Integer.valueOf → Comparable 参数位），与 CharSequence 片的纯引用形多一步装箱，装箱本身是既有域。

## 决策 1：9 行封闭表（String + 8 装箱）

```
java.lang.String     → java.lang.Comparable
java.lang.Byte       → java.lang.Comparable
java.lang.Short      → java.lang.Comparable
java.lang.Integer    → java.lang.Comparable
java.lang.Long       → java.lang.Comparable
java.lang.Float      → java.lang.Comparable
java.lang.Double     → java.lang.Comparable
java.lang.Character  → java.lang.Comparable
java.lang.Boolean    → java.lang.Comparable
```

（release 8 javadoc 的 java.lang 实现者全集；`java.math`/`java.util.Date` 等其余 JDK 实现者**不入本片**——未实测先不扩，task 1.1 核对 javadoc 逐行转录。）落点与 CharSequence 片并列（同一小函数或同一表族），实现者按两片合并派发时的最小结构定。

## 决策 2：命中即 cast_argument，与三条姊妹通道逐字同构

呈现 `(java.lang.Comparable) "a"` 形 cast——与集合/Throwable/CharSequence 命中同一函数同一行为。

## 决策 3：零回退与负例

- 三条既有姊妹通道（java.util/Throwable/CharSequence——若已落地）测试全绿；
- 负例：`Object`→`Comparable`（Object 不实现 Comparable）仍拒；用户类→`Comparable` 走 snapshot（既有域）；
- corpus 双腿扫描：预期 diff 为空（如实记录）。

## 验证标准（可证伪）

1. 主锚 `RG.callGen`/`callGen2` 恢复、整类 `javac --release 8` exit 0、`main` 行为一致；
2. 零回退/负例如上；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint。

## Open Questions

1. 与 CharSequence 片的表结构共享（单函数双表 vs 表族）——合并派发时实现者定，报告说明；
2. `java.math.BigInteger` 等 JDK Comparable 实现者是否同片补——**否**（本片严格 java.lang，其余待实测另片）。
