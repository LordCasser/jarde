# Tasks

- [x] 1. javap 核对 release-8 java.time header 行集（LocalDateTime/LocalDate/LocalTime/Instant/ZonedDateTime → Temporal/TemporalAccessor 等），转录 widening-row-sources 协议证据
      → 实测（`javap` on Corretto 1.8.0_432 `rt.jar` sha256 b27515a6…复核一致）：**十二行**转录（七型 + `Temporal`/`TemporalAccessor` + `CompletableFuture`/`CompletionStage`/`Future`），
      `javap-headers.txt` 末尾续录、`widening-row-sources/README.md` 增行集表；**七型都直接声明 `Temporal`**，`TemporalAccessor` 行经 `Temporal` 自身 header 的
      `extends TemporalAccessor` 到达（链逐型相同、已写出），故扩展四型 8 目全部有 header 依据、无需拒行。自检脚本 `results/selfcheck-javap-rows.sh` 重跑 `javap` 并逐字节 diff（`SELF-CHECK OK`）。
      证据：[results/01-javap-transcription.md](results/01-javap-transcription.md)、[results/javap-raw-lines.txt](results/javap-raw-lines.txt)、[results/selfcheck-javap-rows.out](results/selfcheck-javap-rows.out)。
- [ ] 2. 行集入 `platform_interface_argument_widens` 表族；对照测试：JT/DT 双 javac 腿 + JT.basic 零回退 + 既有表负例
- [ ] 3. 全门禁 + corpus 指纹 + 分逻辑提交（不 push）
