# Root 独立验收（2026-10-06，合并 `659c9c84`）

## 判据逐项

1. **diff 审查**：`319a57e0..82e1f186` 的 `build.rs` 改动 = `TEMPORAL_FAMILY`（14 行）+ `COMPLETABLE_FUTURE`（2 行）两 const 加入既有 `.any()` 链（rustfmt 重排数组字面量为其 3 行删除的来源），release-8 门、`cast_argument` 渲染、既有四表、诊断文本零触碰；表成员单测钉住 16 行 + 22 拒绝对。
2. **行来源 root 复核（不采信转录，live 重跑）**：root 直接对 rt.jar（sha256 `b27515a6…` 已核）重跑 `javap`——七型（LocalDateTime/LocalDate/LocalTime/Instant/ZonedDateTime/OffsetDateTime/OffsetTime）header 逐字声明 `java.time.temporal.Temporal`；`Temporal extends TemporalAccessor`（TemporalAccessor 行 = 经 Temporal header 的链读，`Properties→Map` 同规）；`CompletableFuture<T> implements Future<T>, CompletionStage<T>` 单行双证。转录文件与 live 输出逐字一致。
3. **行为验证（root 本机构建 `659c9c84`）**：
   - `tests/recover_temporal_argument_widening` 3 passed + ignored 回放 1 passed（4.51s；剥离渲染→`javac --release 8` 与真 Corretto 8 双腿→`-Xverify:all` 运行→与原 class 逐项一致：JT `2026-10-06/2026-11-01/09:30`、`2026/01/02 03:04 -> 3:4`、`287`；DT `03:04/03:04`；CF `DATA!`/`fallback:…`/`6`）；
   - root CLI 探针（**jar 输入 + 自述头断言前置**——首次以单 class 输入探针撞上 `environment_policy_snapshot_kind_mismatch` 假零陷阱，按 handoff 纪律以 jar 重做）：JT/CF refusals=0、自述头在；**TWX（MonthDay/Year 表外型）双腿 refusals=2 逐字保留**；
   - corpus：实现片普查（自检正例 4→0/负例 2→2 先行）报告 1987 松散类 0 候选、恰 3 锚移动、6 拒绝句→0、无新增；指纹再生随合并。
4. **门禁（root 合并态实测）**：`cargo test --workspace --all-targets --all-features --locked --no-fail-fast` 两轮完整全绿（**3085 passed / 0 failed / 58 ignored**，318 targets；另两轮各现 1 个已知 flake 家族瞬时失败——run1 `ordinary_generic_projection`（handoff 已载家族，单测复跑两轮 14/14 绿）与 run3 一处未捕获同名族，flake 判定成立）；`cargo fmt --all -- --check` exit 0；CI 逐字 clippy（ci.yml 46-76 提取）`Finished` exit 0；`openspec validate --all --strict` **302/302**。
5. **CI**：319a57e0（twr-javac8 三提交）run 37420206418 四 job success；本合并的 CI 以推送后 run 为准（监控在案）。

## 实现偏差裁定

- 测试文件位于根 `tests/`（`recover_temporal_argument_widening.rs`）而非 `crates/jarde-java/tests/`：**追认**——先例 `recover_platform_implementer_argument_widening.rs` 经 `git log --diff-filter=A` 核实确在根 `tests/`，走 facade `Engine`+`ClassSourceRequest`；表成员单测在 `build.rs` 内与先例同位。
- `TWX` 超出任务书文本的补充负例：**追认**——语料无 java.time/CF 调用点，渲染级负例是行集封闭性的唯一可执行证明。
- CF 恢复后新增两行 `// jarde: omitted physical lambda helper …` 引注：折叠伴生的预期面，非渲染回退。

## 残余边界

- 行集封闭十四行 + 双行；`java.time.chrono.*`、adjuster 类型、`Year`/`MonthDay` 等其余平台实现者不入表（表外型保持拒绝，TWX 已钉）——扩行需新 patrol 证据 + javap 转录，不是本片遗留缺陷。
- root 首次 TWX 探针的假零再次实证 handoff 陷阱 #7：**计数前必须断言自述头**。
