## 1. 取证
- 1.1 复跑 patrol fixture（SA/SD），确认三锚可编译错、postOther 不可编译
- 1.2 Q-i/Q-ii 插桩（结论写入 instrumentation.md）

## 2. 实现
- 2.1 旧值 store 证明失败 → 整条语句（iinc 呈现/数组存）并入引注区
- 2.2 健康形与现有拒绝文本零回退

## 3. 验证
- 3.1 三锚：去注释文本不可编译或行为一致（实证 javac+java）
- 3.2 全门禁 + fixture 双协议（人口计数 + corpus 指纹再生）
