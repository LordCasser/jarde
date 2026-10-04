## 1. 取证与基线（root 已完成大半）

- [x] 1.0 双腿指令级/异常表级几何差、行变体排除、guard CFG 驱动性、MVP 边界（单资源先行）——全部实测并归档于 [TWR 巡查](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/README.md) 各节。（root 已完成）
- [ ] 1.1 **插桩定夺（不得预设）**：Q-i 第一道门（`jre_region_exception_edge`/`jre_region_uncovered_blocks` 的发出判据与 guard `fn guarded` 的到达序）；Q-ii 方向 A/B（既有六项机制哪些在 javac 8 形不成立、能否只加严不放宽）；Q-iii 呈现路径无需改动即可同构。转录存证据目录，**据此选向并在报告说明理由**。
- [ ] 1.2 重验基线：主线二进制渲染 `TR` 双腿（真 javac 8：`one()` 拒 + 6 引注；javac 23：恢复 + 0 引注），与巡查记录一致；`TwrAudit` 行为不变。
- [ ] 1.3 冻结负例探针（合成 classfile 或裁剪源）：削一条 any 行 / ifnull 目标互换 / 抑制链断头——各验证**现状即拒**（作为"新判据是必需的"对照）。

## 2. 实现（按 1.1 结论）

- [ ] 2.1 按 Q-i 落点实现（region 层放行抑制链块 / guard 层几何扩展，或两者），方向 A 或 B 按 Q-ii；**既有 javac 9+ 判据逐字不动**（决策 2）。
- [ ] 2.2 新几何事实以结构事实表达（块集/行集），不认字节码序列字面量；承载方式按 design Open Question 2 定，不放宽既有字段语义。
- [ ] 2.3 无版本判据（决策 4）；插桩行移除并 grep 零残留。

## 3. 验证与验收

- [ ] 3.1 主锚：`TR.one()` 双腿同构恢复（`try (TR local1 = new TR(arg0)) { … }`）、0 引注、整类渲染源集 `javac --release 8` exit 0、`java -Xverify:all` 输出与原 class 逐行一致（真 javac 8 腿修复前 6 引注/拒）。
- [ ] 3.2 零回退：javac 23 腿逐字节不变；`Shape::Resources`/finally 家族测试全绿；`TwrAudit` 不变；corpus 双腿扫描差异仅真 javac 8 TWR 形。
- [ ] 3.3 负例三条各保持拒绝（与 1.3 冻结态一致）。
- [ ] 3.4 门禁全量（基线以合并态为准；flake 单测复跑两轮判定）+ fmt + CI-exact clippy（ci.yml 46-76 逐字）+ openspec strict + `git diff --check` + 再生 corpus fingerprint。
- [ ] 3.5 root 独立复核：插桩转录与选向理由、既有判据零放宽（diff 逐条）、主锚/零回退/负例实测、无版本判据；更新 CF-17 账本（真 javac 8 腿从缺口改为已修，指向本片）与 dual-javac-sweep P08 行。（留 root）
