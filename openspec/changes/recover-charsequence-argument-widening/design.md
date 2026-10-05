# Design：CharSequence 实参扩宽的第三张平台小表

## Context（root 已实测/读码）

- 拒绝点：实参呈现 `java.lang.String`、调用点参数声明 `java.lang.CharSequence` → `cast_argument` 前的三条通道全不命中。
- 既有通道与性质：`platform_reference_argument_widens`（java.util 集合树封闭边表，"one row per documented direct extends/implements"）、`java_lang_throwable_widens`（java.lang Throwable 族）、`snapshot_hierarchy_widenings`（参数快照自证链）。CharSequence 属 javadoc 级平台事实，与前者同性质。

## 决策 1：封闭实现者表（三行），与集合表同构

```
java.lang.String        → java.lang.CharSequence
java.lang.StringBuilder → java.lang.CharSequence
java.nio.CharBuffer     → java.lang.CharSequence
```

（release 8 javadoc 的 CharSequence 实现者全集：CharBuffer、Segment(9+)、String、StringBuffer、StringBuilder——release 8 无 Segment，故三行 + StringBuffer 共**四行**；实现者以 javadoc 为准逐一核对，勿凭记忆，`StringBuffer` 勿漏。）表放 `platform_reference_argument_widens` 内并列（或同文件小函数 `java_lang_charsequence_widens`，实现者按两表共享程度定——若 Throwable 是独立函数则照抄其形）。

## 决策 2：命中即 cast_argument 呈现，语义与两条姊妹通道逐字同构

呈现 = 显式 cast（`String.join((java.lang.CharSequence) "-", xs)` 形）或既有等价形——与集合/Throwable 命中的呈现**同一函数同一行为**，无第三种呈现。

## 决策 3：零回退与负例

- 集合扩宽既有测试（`recover-platform-collection-widening` 家族）与 Throwable 通道零回退；
- 负例：`String` → 非 CharSequence 接口（如自定义接口或 `Runnable`）仍拒；`Integer`→`CharSequence`（非实现者）仍拒；
- corpus 双腿扫描：预期 diff **为空**（语料 0 真实 String.join 调用——如实记录；空 diff 是回归证据）。

## 验证标准（可证伪）

1. 主锚 `SB.join` 恢复（0 引注）、整类 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致（含 join 段 `p-q`）；
2. 负例两条仍拒；姊妹通道测试全绿；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint。

## Open Questions

1. 表的落点（`platform_reference_argument_widens` 内 vs 并列函数）——实现者按 Throwable 先例定；
2. `(CharSequence) "-"` cast 呈现是否会破坏 `String.join` 的可读性——按既有 cast_argument 形即可（全仓一致）。
