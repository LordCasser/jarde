## 当前状态

- 主线f4a81440d已交付FO/loopless；旧patrol矩阵保留为历史。
- noCall产品02de294f7的本地最终CLI重筛：FY/FO各5方法均structured/java、无fallback，完整源码两种javac原样重编与main双JVM共8次观察一致；自身CI38872425446已终态独审通过，并已正常归档交付main e00c48c44928bf051377f23cbf9244f378a61a5a；父剩余任务不由子片替代。
- CallReturn叶片最终CLI b4a6b2ce…c9c851已恢复FX.finReturn为structured/java，FX两份完整源码现在原样重编成功；FY/FX/FO共6完整类/12 main输出等原始基线。FX保留6方法，finContinue/finBreak仍有jre_region_uncovered_blocks handler45（注释45/47/50/52）。固定main匹配不等于handler已恢复；父全锚/JADX/逐字节对照仍未完成。CallReturn自身CI38913819742已终态独审通过（双seed各3670 passed），已正常归档；父全锚/JADX缺口不由该CI代替。
- 固定JADX1.5.6最新三方观察：FY/FO两腿完整源码重编0、8 main匹配；FX两完整输出因未定义th重编1，无FX重建运行。JADX包改为defpackage按实记录；不修文本、不以其它工具失败证明jarde语义完整，父全锚/handler仍待办。
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
