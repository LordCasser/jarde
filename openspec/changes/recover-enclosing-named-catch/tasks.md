## 1. 基线与负例

- [x] 1.1 重放固定 T3/T1/C4：核 fixture SHA、主线 `jre_guard_unexplained_row` 基线与实验捕获（放宽后 `[38]` 未覆盖）；记录可重放基线。
  - fixture SHA 复核一致（巡查 `fixture-sha256.txt` vs 冻结字节；C4 = cf15 冻结件）。主线基线：五固定形状整方法回退 `jre_guard_unexplained_row`@0（T3.voidNamed 现场重放与巡查 base JSON 一致）；正例族 W17b 五成员同因回退（`enclosing-17b/results/W17b-*.base.json`）。
- [x] 1.2 构造并冻结至少五个 verifier 有效负例：catch-all 包围行（保持 Unexplained）、包围行只覆盖部分跨度、handler 块与 claim 交叠、handler 体含分支/循环（保持拒绝）、双 catch 子句（保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。
  - `enclosing-17b/original/N17b.class`（[N17bGen.java](../../evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/enclosing-17b/N17bGen.java) 可复现）：六形状（第五、六为双 catch 与 multi-catch 各一），本体全部 verifier 有效可运行（`done`×6，测试钉死）；实现前（stash 主线二进制）与实现后 fallback 列表逐字一致——五例 `jre_guard_unexplained_row`（+uncovered），部分跨度例 `jre_region_uncovered_blocks`（记录于 `results/N17b-*.{base,after}.json` 与 README 映射表）。

## 2. 容忍与子句呈现

- [x] 2.1 `enclosing_clauses` 全跨度具名行分支（design 决策 1），守卫齐备；T3 两形 guard 通过；部分覆盖分支与 catch-all 拒绝逐字不变。
  - 分支守卫：具名类型 + `start <= shape_start && end >= handler_end` + handler 在 claim 外 + 至多一行全跨度行（两行即双 catch/multi-catch 保持拒绝）+ handler 单直行块证明（绑定 store、语句白名单 + 值 return、唯一 normal 后继 = 语句续块，`goto` 桥折叠）；注释按 design 改写。部分覆盖与 catch-all 走原路径，负例逐字不变。
- [x] 2.2 Plan 携带包围子句、Builder 发射 `catch (E e)` 与 handler 体呈现、覆盖账本记入（design 决策 2）；T3/T1.twrNamed/C4.twrNamed 完整恢复（叠加形状含 17a）；预算/取消原子回滚。
  - `Plan.enclosing: Option<Box<EnclosingCatch>>`；clause handler 入口入 `plan.facts`（catch 读者让位）；handler 块入 `plan.owned`（覆盖账本）；Builder 在 TWR 头后发射子句（头拼写复用 `catch_header` 路径，体走普通语句通道，绑定 store 跳过、参数槽永不切分）。T3 两形、T1.twrVoidNamed/twrPopNamed、C4.twrNamed、`Guarded.withCatch` 恢复；无 catch TWR 家族、Catches 既有路径、finally 证书零回退（全仓测试 + p5 钉值）。预算/取消：紧上限回答 Incomplete 不发布任何内容，取消令牌零发布（测试钉死）。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（TWR 家族、Catches 既有路径、17a 切片零回退）、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘纪律同前。
  - 全仓 265 个测试目标 `test result: ok`（各轮 `df -h /` ≥15Gi）；`cargo fmt --all -- --check` 干净；clippy 逐站点配对：新增 0（本地 1.98 存量 91 站点持平，`Verdict` 装箱避免 large-variant 新告警）；`openspec validate --all --strict` 225 项全过；四条 ignored 失败（d5 chainCast、jdk25、Combo 未来验收、release 构建）经 stash 对照为主线既有。
- [x] 3.2 T3/T1/C4 三方对照：原 class/固定 JADX Java-input/Jarde `javac --release 8` 重编，`java -Xverify:all` 正常路径与注入异常路径（ISE 被 catch、清理异常传播、suppression 保留）逐路径一致；记录输出 SHA。
  - `enclosing-17b/results/three-way/`：T3/T1/C4 三腿逐行一致；W17b 五路径（正常、体注入 ISE、handler 调用体、清理异常传播、suppression）jarde 与原类逐行一致，JADX 列 `closeThrows` 一处自身重构缺陷（close 移入保护区并虚构 suppression，与 17a 记录同类）——`run-sha256.txt`/`leg-source-sha256.txt` 落盘。恢复文本 `javac --release 8` 一次通过，无机械补丁。
- [x] 3.3 root 独立复核容忍守卫、子句呈现与覆盖账本，更新 CF-17 清单与巡查账本。（root 于合并主线 5644fc85 复核：T3 两形、T1、C4.twrNamed 与 Guarded.withCatch 全部恢复为 `try (…) { … } catch (E e) { … }`、六负例逐字保持拒绝、全仓 2723/0〔首跑 1 例 plugin plane flake 重跑干净〕、fmt/openspec 225/225。至多一行全跨度行守卫与单直行 handler 证明复核通过；handler 参数传递槽复用限制登记 Catches 扩展片域。）
