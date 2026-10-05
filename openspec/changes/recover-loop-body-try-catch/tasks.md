## 1. 取证与基线

- 1.1 复跑 loop-body-catch-patrol fixture（DF/DV），确认三形拒+两对照恢复与 README 记录一致
- 1.2 回答 Q-i/Q-ii/Q-iii（插桩结论写入本目录 `instrumentation.md`）

## 2. 实现

- 2.1 按 Q-i 结论扩展现有机制（proved_loop_catch_joins 或 fragmented 认证路径）覆盖简单形；结构判据零放宽（引用逐字判据）
- 2.2 对照组零回退：无 try 循环、循环内 try-finally、fragmented-loop-catches 既有 fixture 逐字节不变

## 3. 验证

- 3.1 三方对照：jadx（已知错码）vs jarde 新输出 vs 源——jarde 必须行为精确（{1,-2,3}→3 等）
- 3.2 全门禁：workspace tests 基线不回退；fmt/clippy 按 CI 逐字
