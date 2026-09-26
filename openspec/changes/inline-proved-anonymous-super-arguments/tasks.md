## 1. 冻结输入与拒绝边界

- [x] 1.1 冻结无捕获 DT-06a 的原/JADX/Jarde 三方完整源码、3 个 class SHA 和 `javap`；root 独立执行 `python3 openspec/evidence/java-syntax-2026-09-27/anonymous-super-direct/replay.py`，确认三方 Java 8 重编、`-Xverify:all` 均输出 `13:2`，旧 Jarde 保留物理 `$1` 使用点。
- [x] 1.2 构造构造器转发 slot/目标重载错误、捕获字段或多余效果、同类第二 BCI、跨类身份引用的最小负例；集成测试逐项断言拒绝时保留物理构造及独立子类报告。

## 2. 父类构造与匿名类族证明

- [x] 2.1 在 DT-05 的 typed 匿名关系和完整 owner XRef 路径上接纳零接口、非 `Object` 父类的准确单站点候选；用正例、第二 BCI、跨类引用和不完整扫描测试核对身份门槛。
- [x] 2.2 从同次构造器 AST/prologue、SSA effect 和完整原始 Code 证明每个物理参数按 descriptor slot 原样且有序传给唯一所选父类 `<init>`，无字段写入/分支/处理器/额外效果；用 `(II)V` 与 `(IJ)V` 重载及错位/捕获负例核对拒绝。
- [x] 2.3 核对父类及构造器在所选环境准确解析、同包源码可访问，子类全部需写方法为完整 Java 8 正文且父类成员调用可写；缺目标、fallback/Mixed、错误声明或低预算/取消的测试不得产出匿名源码。

## 3. 原子 AST 发射与三方验证

- [x] 3.1 复用准确 `New` 节点，把原调用者 AST 的实参按原序写进 `new Base(args...) { ... }`，从子类同次 AST 写全部方法；测试断言 `next()` 两次、`super.sum()`、创建/参数 BCI 和不同 evidence 选项的文本一致。
- [x] 3.2 只在类族证明与输出预算全部完成后发布根类源码，任何拒绝保持原物理构造和返回类型；扩展重放脚本的独立 fixed 输出，按 Java 8 重编根类加真实 `Base`、`-Xverify:all` 验证 `13:2`，物理 `$1` 另查。

## 4. 架构师独立验收

- [x] 4.1 Root 独立复跑冻结三方与 fixed 回放、定向及 DT-05 回归、`cargo fmt --all -- --check`、`openspec validate inline-proved-anonymous-super-arguments --strict`；复核身份、参数次序、预算、来源和原子拒绝后，才更新 DT-06 窄切片状态并提交推送。
