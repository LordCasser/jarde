# Tasks

- [x] 1. javap 核对 release-8 java.time header 行集（LocalDateTime/LocalDate/LocalTime/Instant/ZonedDateTime → Temporal/TemporalAccessor 等），转录 widening-row-sources 协议证据
      → 实测（`javap` on Corretto 1.8.0_432 `rt.jar` sha256 b27515a6…复核一致）：**十二行**转录（七型 + `Temporal`/`TemporalAccessor` + `CompletableFuture`/`CompletionStage`/`Future`），
      `javap-headers.txt` 末尾续录、`widening-row-sources/README.md` 增行集表；**七型都直接声明 `Temporal`**，`TemporalAccessor` 行经 `Temporal` 自身 header 的
      `extends TemporalAccessor` 到达（链逐型相同、已写出），故扩展四型 8 目全部有 header 依据、无需拒行。自检脚本 `results/selfcheck-javap-rows.sh` 重跑 `javap` 并逐字节 diff（`SELF-CHECK OK`）。
      证据：[results/01-javap-transcription.md](results/01-javap-transcription.md)、[results/javap-raw-lines.txt](results/javap-raw-lines.txt)、[results/selfcheck-javap-rows.out](results/selfcheck-javap-rows.out)。
- [x] 2. 行集入 `platform_interface_argument_widens` 表族；对照测试：JT/DT 双 javac 腿 + JT.basic 零回退 + 既有表负例
      → 落表 `TEMPORAL_FAMILY`（14 行）+ `COMPLETABLE_FUTURE`（2 行），同函数同门同 `cast_argument` 呈现；diff 纯增（222+/3-，3 行删除是 `.any()` 链数组重排=membership 本体，如实披露）。
      冻结锚（巡查 fixture 源逐字节复制 + 本片负例 `TWX`，两条 javac 腿）：`tests/fixtures/recover-temporal-argument-widening/`，
      **`v8` 腿 class 与巡查 jar 逐字节相同**（`cmp` 实测）。对照测试 `tests/recover_temporal_argument_widening.rs`（3 passed + 1 ignored）：
      `JT.fmt`/`JT.spans`/`DT.direct`/`CF.combined` 恢复 0 引注、`JT.basic`/`DT.parse`/`CF.chain`/`CF.recover` 整段文本逐字节不变、
      `TWX` 两条拒绝逐字保留（引注计数 = 2）；`build.rs` 单元测试钉 16 条新行 + 22 条表外/release/形状负例。
      行为验收（剥离→装机 `javac --release 8` 与真 javac 8 双编译→`-Xverify:all` 运行）：与 fixture 自身 class 输出逐字节一致
      （`JT` `2026/01/02 03:04 -> 3:4`、`287`；`DT` `03:04/03:04`；`CF` `DATA!`/`fallback:…x`/`6`）。
      证据：[results/02-rows-anchors-tests.md](results/02-rows-anchors-tests.md)、[results/02-acceptance-roundtrip.out](results/02-acceptance-roundtrip.out)、[results/corpus-sweep.out](results/corpus-sweep.out)。
- [x] 3. 全门禁 + corpus 指纹 + 分逻辑提交（不 push）
      → 门禁：`cargo fmt --all -- --check` exit 0；CI 逐字 clippy（46–76 行 + `-D warnings`）exit 0；
      `cargo test --workspace --all-targets --all-features --locked` = 318 目标 / **3085 passed / 0 failed / 58 ignored**；
      `openspec validate --all --strict` = 301 passed / 0 failed。
      语料：指纹再生 **+12 条目**（4 源 + 8 class，0 删除）；fixture 人口 `(691,2943,282,1827,8)` → **`(699,2989,282,1827,8)`**（+8 类/+46 body）。
      增量分类：渲染面只有本片三锚移动（引注 6 → 0，新增 0），散装 1987 class 候选 0，既有测试 pin 0 处移动。
      证据：[results/03-gates.md](results/03-gates.md)。
