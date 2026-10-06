# Tasks

> 纪律：落点钉定前必须做最小门控实验（BCI 56 的 join 为何被双主张——插桩或诊断逐字定位产生路径）；行号按锚点名重验。

- [ ] 1.1 冻结锚与对照（复用巡查 fixture：`BS.java` 全类、`CB.java` 四形矩阵；双腿真 javac 8 + `--release 8`）；渲染基线：`loopElseIfRet`/bsearch canonical 拒绝、三对照恢复；诊断逐字记录。
- [ ] 1.2 插桩定位：BCI 56 join 的两个主张者各是谁（走查的区域序列 + 各区域 blocks）；阶梯早退臂的 return 叶为何使 join 被二次主张；与已恢复的 `loopIfElseRet`（单 if-else+早退）的选举差分钉死判别变量。
- [ ] 2.1 实现 join 选举修正（阶梯=单一 if-else 树，早退臂为终止叶）；单臂形判据零改动。
- [ ] 2.2 对照测试：锚恢复（0 引注、重编 exit 0、`-Xverify:all` 行为一致）；三对照逐字节零回退；负例（异常表交叉、双层阶梯）保持拒绝。
- [ ] 3.1 全门禁 + corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：插桩定夺、判据零放宽 diff 逐条、锚/对照/负例实测、账本更新（第 5 族 canonical-overlap 可恢复性锚关闭）。（留 root）
