## 1. 取证与基线
- 1.1 复跑 patrol fixture（SL/SM/SN），确认矩阵与 README 一致
- 1.2 回答 Q-i/Q-ii（插桩结论写入 instrumentation.md）

## 2. 实现
- 2.1 MVP：case 内 return 恢复（按 Q-i 落点，判据零放宽）
- 2.2 第二步：case 内 break outer 标签重建（等价证明）
- 2.3 对照零回退：SM 五对照形逐字节不变

## 3. 验证
- 3.1 三方行为：SN 四锚 + 判别输入 {9,1,5}→10；jadx labBreak 已知错码为反例
- 3.2 全门禁：workspace tests 基线不回退；fmt/clippy CI 逐字
