## 1. Group same-method enum projections

- [ ] 1.1 移除同方法多站点的 blanket refusal；对候选引用的全部不同表实现严格、完整、一次的共享 helper `<clinit>` 联合证明，不扩展候选发现或依赖读取。
- [ ] 1.2 将每个站点的标签编辑应用到同一份恢复方法 AST 副本，验证各站点 BCI 和 selector，只发射一次完整方法体，并在整组成功后暂存。
- [ ] 1.3 保持逐站点证明证据、标记和拒绝原因准确；额外表/未知操作及任一证明、AST 形状、输出预算或取消失败时，整方法保留原整数路径。

## 2. Verify the bounded slice

- [ ] 2.1 增加 DT-31 双站点正例：同一方法含不同 enum 类型和 table；断言两个 switch 均输出 enum 标签，且完整源码通过 `javac --release 8`。
- [ ] 2.2 使用 `java -Xverify:all` 对原始与重建源码运行所有 enum 值及每个 null selector；比较返回值、异常类别和可观察副作用顺序。
- [ ] 2.3 增加双站点部分证明负例：使一个站点的 map proof 确实拒绝，断言两个站点都没有投影，且拒绝原因关联至失败候选。
- [ ] 2.4 保持单站点投影逐字节稳定；验证独立方法恢复仍保留整数 case；比较 essential/all 成功文本，并覆盖取消或输出预算不足时的原子发布。
- [ ] 2.5 运行定向 Java/class-source 测试、格式与必要仓库检查；审阅证明记录和完整源码 replay。
