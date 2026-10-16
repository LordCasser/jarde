## 当前状态

- 主线f4a81440d已交付FO/loopless；旧patrol矩阵保留为历史。
- noCall产品02de294f7的本地最终CLI重筛：FY/FO各5方法均structured/java、无fallback，完整源码两种javac原样重编与main双JVM共8次观察一致；自身CI尚待终态验收/主线交付。
- FX保留6方法，finContinue/finReturn/finBreak仍mixed/fallback，finReturn缺return造成两份完整源码编译失败。父级完整FX/JADX/逐字节对照仍未完成。
- 各checkbox按原验收范围判断：已交付子片不代替整条父任务；设计与父级instrumentation文件缺失不伪称已补齐。

## 1. 取证与基线
- [ ] 1.1 复跑 patrol fixture（FX/FY/FO），确认矩阵与 README 一致
- [ ] 1.2 将历史Q-i与loopless归档对应，记录noCall实际证据及finReturn当前Q-ii/剩余边界（instrumentation.md）

## 2. 实现
- [ ] 2.1 补齐单return穿副作用finally叶片：noCall由子项承接，finReturn仍保留父待办；复用现有Finally/SavedReturn，判据零放宽
- [x] 2.2 FO 求值序判别测试入测试集（1/100）：recover_static_field_prefix_return的三项永久测试与已归档完整原样源码/driver证明先保存再finally；本片续跑该target为3 passed，20命令整类重筛另有FO双编译器/双JVM固定main输出1/100。
- [ ] 2.3 对照零回退：emptyFin/finContinue/finBreak/finally-return-value 逐字节不变

## 3. 验证
- [ ] 3.1 三方行为：FY/FX 全锚 + FO 判别；jadx 对照
- [ ] 3.2 全门禁：workspace tests 基线不回退；fmt/clippy CI 逐字
