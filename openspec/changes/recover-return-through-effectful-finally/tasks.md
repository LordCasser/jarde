## 1. 取证与基线
- 1.1 复跑 patrol fixture（FX/FY/FO），确认矩阵与 README 一致
- 1.2 回答 Q-i/Q-ii（插桩结论写入 instrumentation.md）

## 2. 实现
- 2.1 MVP：单 return 出口 + 单副作用 finally（temp 绑定形，判据零放宽）
- 2.2 FO 求值序判别测试入测试集（1/100）
- 2.3 对照零回退：emptyFin/finContinue/finBreak/finally-return-value 逐字节不变

## 3. 验证
- 3.1 三方行为：FY/FX 全锚 + FO 判别；jadx 对照
- 3.2 全门禁：workspace tests 基线不回退；fmt/clippy CI 逐字
