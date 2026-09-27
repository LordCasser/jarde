## Why

固定 JADX `TestTryCatchFinally12.TestCls` 的 `test1/2/3` 覆盖内层具名 catch 与外层 finally、共用正常汇合点和异常重抛，九条运行断言检查副作用顺序。原 class 与固定 JADX 完整源码一致；当前 Jarde 三个方法均因 `jre_guard_finally_copy` 只给字节码引用。现有共享 finally 证书只处理静态调用、静态整数字段增量或当前实例布尔赋值，不能证明 `StringBuilder.append` 的两份或三份副本。

## What Changes

- 在已有 Guard/Region/Builder 恢复链中，证明受限 `this.sb.append("-finally")` 的每份物理副本、异常表行、正常出口与原异常重抛；输出一次 Java `finally`，并让 `test1/2` 的内层具名 catch 保持在外层受保护正文内。
- 保留所有物理 BCI、异常表和成员身份来源；常量、接收者、目标成员、异常覆盖或出口不等价时原子拒绝，不发布部分折叠。
- 用固定方法字节码和去除无关 `runTest` 的完整 Java 8 对照类，核九条路径的原/JADX/Jarde 运行结果，并回归已验收的 CF-16 形态。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明的嵌套具名 catch 与两份/三份相同实例追加清理，恢复语义等价的 Java `try`/`catch`/`finally` 与完整物理来源。

## Impact

主要影响 `crates/jarde-java` 既有 guarded-region 证明、Region 有界正文和 Builder 的 `finally` 构造，以及定向 Java 8 fixture/测试。无需新 crate、公共 pass 或运行时依赖。固定类的 `runTest(II)String` 另在 BCI 40 有 switch/try Region 多 owner 拒绝，InnerClasses family 输出也未闭合；二者单独记录，不混入本变更或据此宣称整份固定类已经追平。`FinallyOnce.handled/escaping`、任意清理语句、清理覆盖型完成、TWR 与同步块均不在此范围。
