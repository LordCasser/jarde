## 1. 冻结输入与拒绝边界

- [x] 1.1 冻结顶级普通枚举的 0、1、4 常量 Java 8 源、全部 class SHA、原/JADX/Jarde 全源码及完整重编运行脚本；root 独立复跑 `JADX=/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx python3 openspec/evidence/java-syntax-2026-09-27/enum-arity/replay.py --expect-jarde baseline`，原/JADX 同为 `empty=0`、`one=ONLY:0/1`、`four=[NORTH, SOUTH, EAST, WEST]`，旧 Jarde 在 Empty、One、Four 各有一条 `enum constant expected here` 诊断，4/4 SHA 相同。
- [x] 1.2 复用已冻结的标准辅助方法篡改与额外 `$VALUES` 读取反例，再构造隐式构造器多余效果及不完整 Code 负例；定向测试确认这些情形整组拒绝，预算/取消也不发布半个列表。

## 2. 普通枚举类级证明

- [x] 2.1 将普通常量字段和 `$values()` 数组工厂的固定两项比较改为按完整物理表的有界向量证明；定向测试逐项核对 0、1、4 的字段、ordinal、工厂顺序及重复/错误数组元素拒绝，旧两常量测试仍通过。
- [x] 2.2 在既有构造器证书中区分已证的隐式无源参数 `(String,int)V` 与现有源整数参数形态；测试核对完整原始 Code、SSA/effect、无字段写入及异常、错误转发拒绝，且已有委托构造器不退化。
- [x] 2.3 用同次 `<clinit>` AST/Code 和成员使用普查按常量向量证明构造/写入前缀及唯一隐式数组赋值；测试覆盖零常量、四常量、附加用户引用和不完整成员，拒绝时物理报告不变。

## 3. 原子类源码投影

- [x] 3.1 从已证向量生成合法空/单/多常量列表、需要时的分号及用户方法，按构造证书隐藏隐式构造；定向测试断言 0、1、4 的源码形态和物理字段/方法身份、原始恢复文本及来源不变。
- [x] 3.2 仅在全部证明与输出预算成功后替换类源码；执行冻结三方完整源码的 `javac --release 8` 与 `java -Xverify:all`，核对 `values()`、`valueOf()`、身份、顺序及用户方法效果，运行旧两常量、匿名常量体、用户 static 后缀和 `$VALUES` 额外读取回归。

## 4. 架构师独立验收

- [ ] 4.1 Root 独立复跑冻结脚本、定向与受影响回归，审查使用普查/预算/来源/原子发布边界，执行 `cargo fmt --all -- --check`、`openspec validate recover-proved-plain-enum-arities --strict`；通过后更新 DT-10 盘点状态并提交推送。
