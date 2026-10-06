## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、同因判别（与 CharSequence）、装箱参与、javadoc 集初枚举——实测归档。（root 已完成）
- [x] 1.1 javadoc 逐行核对 9 行封闭集（String + 8 装箱；确认无遗漏的 java.lang 实现者），转录依据存证据。
      → 实测（`javap` on rt.jar，转录存 `widening-row-sources/`）：九行全部 header `implements java.lang.Comparable<…>` ✓；
      **偏差如实记录**：`java.lang.Enum` 自身也实现 `Comparable`（其 header 有），故“java.lang 实现者全集”严格说不止九行——本片按 spec 钉的九行落表（`Enum` 不入表，保守），且无锚需要它。
- [x] 1.2 重验基线：主线二进制渲染 RG（`callGen`/`callGen2` 拒、其余恢复）；负例探针（Object→Comparable）现状拒绝记录。
      → 实测：`callGen` BCI 4 / `callGen2` BCI 8 拒（`RG$IntNode.cmp` 健康面已恢复）；`Object` 实参在源级**不可产生**，以单元级表拒绝钉住。

## 2. 实现

- [x] 2.1 按决策 1 落 9 行表（落点与 CharSequence 片并列，结构按 Open Question 1）；命中走 `cast_argument`（决策 2）。
      → 落点：`platform_interface_argument_widens` 内的 `COMPARABLE` 表（与 CharSequence/Serializable/枚举族同函数、同 release-8 门、同 `cast_argument`）。
- [x] 2.2 不动既有三条通道；不做 java.lang 之外的 JDK 实现者。
      → 实测：java.util 表/Throwable 通道/数组闭集逐字未改；`java.math.BigInteger` 负例实测仍拒。

## 3. 验证与验收

- [x] 3.1 主锚：`callGen`/`callGen2` 恢复、整类 `javac --release 8` exit 0、行为一致。
      → 实测：`max((java.lang.Comparable) "a", …)` / `max((java.lang.Comparable) java.lang.Integer.valueOf(1), …)`；`RG` 整类 refusals=0；`CO` replay（双腿）答案 `1/b/2/b` 与 fixture 自身 class 一致。
- [x] 3.2 零回退：三条姊妹通道测试全绿；负例仍拒；corpus 双腿扫描 diff 为空。
      → 既有集合/Throwable/平台接口测试全绿；`COX.big` 仍拒（计数恰 1）；两处既有 pin 因本表**严格更好地恢复**而移动（`same_class_generic_binding` 的 SCGA/SCGF `main`，见本片 verification“移动的 pin”）。
      **语料扫描实测（修正 spec 的“空 diff”预期）**：52 个候选类里 6 类有 diff，全部“拒绝→恢复”（新增行 0 条拒绝句），其中本片相关 = `recursive-generic-patrol/RG`（2 条）、`same_class_generic_binding` 的 `SCGA`（1 条）、`nested-generic-header-patrol/Z1`（2 条，额外收益）、`generic-static-field-init-patrol/RG`（2 条，额外收益）——见 charsequence 片 verification“语料扫描”节。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
      → 见本片 verification“门禁”节。
- [ ] 3.4 root 独立复核：javadoc 集核对、表封闭不外推、零回退实测；关闭 summary.md 登记行。（留 root）
