## 1. 取证与基线

- [x] 1.1 重放 Y1/Y1M（SHA 核对）：两门定位与投影输入集合清单；两案 Y1+corpus 对比；token 锚定门依赖关系取证（design Context (a)–(c)）；记录基线。
- [x] 1.2 冻结至少四个变体/负例：类子+lambda、数组投影根、枚举投影根、无投影根（零回退断言集）；各自 `java -Xverify:all` 前后记录。

## 2. 投影保留与折叠

- [x] 2.1 择案实现（design 决策 1–2）；Y1 族折叠、lambda 内联保留、重编行为一致（`hi!`/`45`/`[b, aa]`/`8`）；重跑不降级测试钉死。
- [x] 2.2 无 lambda 根与全部既有家族 diff 逐字不变；token 锚定门结论如实（独立补丁或登记）；预算/取消不变。

## 3. 回归与验收

- [x] 3.1 全仓测试全绿（含 lambda/折叠/混合全部家族）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [x] 3.2 Y1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核契约变更面、重跑不变量与三方行为，更新账本（嵌套声明里程碑第三层入口）与巡查记录。（root 于合并主线 d2f9c130 复核：Y1 jar 输出双呈现同时成立——`static interface StrFn` 折叠 + `StrFn local1 = (java.lang.String p0) -> p0 + "!";` 内联 + 5 消隐标记；全仓 2902/0、fmt/openspec 261/261；corpus 8 类新折叠全部重编行为一致。`projection_inputs` 可选字段（serde 空-跳过实测零回退）与"改名/数组改写成员需改写时显式拒绝（不回退物理体）"均复核认可；token 健全锚（生产者定义指令 CP 条目、未放宽覆盖段）与 InterfaceMethodRef 直接锚另片边界接受。第二位实现者的 2×2 证伪是本片成立的前提——记入巡查。）
