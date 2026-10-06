# Tasks

- [ ] 1. javap 核对 release-8 java.time header 行集（LocalDateTime/LocalDate/LocalTime/Instant/ZonedDateTime → Temporal/TemporalAccessor 等），转录 widening-row-sources 协议证据
- [ ] 2. 行集入 `platform_interface_argument_widens` 表族；对照测试：JT/DT 双 javac 腿 + JT.basic 零回退 + 既有表负例
- [ ] 3. 全门禁 + corpus 指纹 + 分逻辑提交（不 push）
