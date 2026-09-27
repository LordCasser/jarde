# DT-12 匿名枚举常量体审计

## 固定输入与 JADX 证据

基线为本地 JADX 1.5.6，提交 `2fb1b16386941660fda07e9017285aec40fcb37f`，从该 checkout 的 `jadx-cli/build/install/jadx/bin/jadx` 执行。样例和算法源的 SHA-256 在 [source-sha256.txt](source-sha256.txt)，本目录保存缩小后的 Java 8 可执行 fixture；[replay.py](replay.py) 从原始 fixture 编译 `-g` 与 `-g:none` 两套 class，完整反编译源并在临时目录以 `javac --release 8` 和 `java -Xverify:all` 对照原文、JADX 与 Jarde。运行形式为 `python3 replay.py baseline /absolute/path/to/jarde-cli`；baseline 模式要求 `TestEnums2a` 的 Jarde 全类 Java 8 重编失败并出现冻结的枚举语法诊断。未来修复后用 `fixed` 模式，结果单独保存在 `fixed/`，不覆盖基线；它要求该类重编成功且 `-Xverify:all` 行为逐字匹配。`TestEnums6` 只是相邻观察项，不作为这项变化的准入门。

`enums/TestEnums2a.java` 的唯一相关枚举 `DoubleOperations` 实现 `IOps`，包含 `TIMES("*")`、`DIVIDE("/")` 两个匿名常量体，各自覆写 `apply(double,double)`；公共枚举构造器消费一个显式 `String` 参数并存入 `op` 字段。JADX 测试只断言输出片段 `TIMES("*") {` 和 `DIVIDE("/")`，没有调用文件中定义的 `check()`。本审计的 runner 因此另外核对 `getOp()`、运算结果与匿名运行时类名，避免只把文本相似当成功。

`enums/TestEnums6.java` 测的是另一条边界：`ZERO` 与 `ONE(1)` 没有匿名常量体；无参构造器委托到 `Numbers(int)`。其测试只做 `ZERO,`、`Numbers() {`、`ONE(1);` 片段断言，也没有调用 `check()`。这是 DT-11 枚举构造器/普通常量形态，不是 DT-12 匿名常量体正向样例。保留它是为了固定并纠正账本的错误合并。

JADX 的 `EnumVisitor.processConstructorInsn` 对非主枚举构造 owner 调 `processEnumCls`，随后为解析到的枚举构造器标记需从源实参跳过的 JVM 隐式 name/ordinal。`createEnumFieldByConstructor` 将其余实参恢复为常量构造实参。`ProcessAnonymous.canBeAnonymous` 与 `checkUsage` 再判断该合成类能否内联。可借鉴的是从常量构造点关联专属类体并在枚举常量位置输出；单凭类名/synthetic 标记或“构造器只在一个方法使用”不足以证明类身份及构造桥语义。现有 Jarde 枚举体 OpenSpec 已以 ordinal 转发反例冻结这一风险，见[桥接控制分析](../../java-syntax-2026-09-25/enum-constant-body-bridge-controls/analysis.md)：字节码改写保持 verifier-valid，而 JADX 输出运行时 ordinal 错误。

## Jarde 当前证明范围

现有 [匿名常量体证明](../../../changes/recover-proved-enum-constant-bodies/proposal.md)成功覆盖冻结的零源参数 `Op`/`Mixed` 子类形态。当前同次候选证书在 [facade.rs](../../../../src/facade.rs#L9284) 明确要求 `descriptor_source_argument_count == 0` 且主构造 descriptor 为 `(Ljava/lang/String;I)V`。这证明零源参数切片已实现，不证明任何显式 enum 常量构造实参可与匿名体原子投影。

## 三方执行结果

`replay.py` 对 `TestEnums2a` 和 `TestEnums6` 分别运行 `-g`、`-g:none` 两组完整类。每组原始 Java 8、JADX 全源码均编译成功，并通过 `-Xverify:all`；两边运行结果一致：

```text
TestEnums2a: TIMES=*:6:demo.DoubleOperations$1
             DIVIDE=/:2:demo.DoubleOperations$2
TestEnums6:  values=ZERO:0,ONE:1
             declared-constructors=[2, 3]
```

Jarde 保留了 `DoubleOperations` 的普通物理字段和指向 `DoubleOperations$1` 的桥式构造器文本，完整源码 `javac --release 8` 因 `enum constant expected here` 及物理匿名子类类型无法作为源构造参数而失败；`Numbers` 同样以普通 `ZERO` 字段而非枚举常量输出，故 Java 8 重编失败。两组 Jarde 失败都没有运行 class，不将其称为语义不等价执行。完整逐模式源码、javac 输出、`javap`、原 class hash 和工具链记录均在 `outputs/`。

## 结论与切片边界

DT-12 有具体缺口：JADX 对 `TestEnums2a` 的单个 `String` 源构造参数与两个匿名常量体成功输出，而 Jarde 当前零源参数证书不准入此形态。新提案只计划扩展到这个闭合组合：两个有序常量、每个一个字符串字面量实参、共享单一 `String` 主构造参数与现有无捕获匿名体关系。任意参数类型/表达式、不同数量常量、多个构造器、捕获、额外桥转发和其它匿名子类不在提案内，证据不足时继续保留物理事实并拒绝该源码投影。

`TestEnums6` 回到账本中 DT-11 的相邻边界，不因本次重放将 DT-11 的 String varargs 需求并入此提案。整个审计没有更改生产代码。
