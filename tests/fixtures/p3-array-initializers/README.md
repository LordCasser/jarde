# P3 fixture：数组初始化链

本目录固定了 Java 8 一维数组初始化器样本。`v8/ArrayInitializerProbe.class` 是真实编译产物；旁边的 `ArrayInitializerProbe.java` 和 `ArrayInitializerRunner.java` 与冻结审计输入逐字节相同。编译器仅用于生成/复现输入，测试运行时直接读取已提交的 class。

| 属性 | 值 |
| --- | --- |
| 类 | `ArrayInitializerProbe`（默认包） |
| class 文件版本 | 52.0（Java 8） |
| class 大小 | 1281 B |
| SHA-256 | `998bdb54c863d92cb63cc08c654df21bc674961c652fc46a5c3eaf7de2915c86` |
| Code 属性 | 13 |
| 编译器 | javac 23.0.1（OpenJDK 23.0.1） |
| 编译命令 | `javac --release 8 -g:none -d v8 ArrayInitializerProbe.java` |
| 调试属性 | 无（`-g:none`） |

证据来源为 `openspec/evidence/java-syntax-2026-09-22/array-initializers/`。其原 class、源码和 runner 的 SHA-256 分别为 `998bdb54…2915c86`、`573cde1a…db9ac71`、`1837934e…e5f802`；本 fixture 复制后逐一复核一致。冻结 CLI `7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34` 的结果记录在该证据目录，没有把 CLI 输出或临时产物复制进 fixture。

## 样本覆盖

`literalInts` 与 `literalStrings` 固定非空基本类型及引用类型初始化；`emptyInts` 与 `emptyStrings` 固定现有空分配行为；`effectfulInts` 与 `effectfulStrings` 固定元素从左到右只求值一次，以及第二/第三个元素抛异常时已发生的 trace。runner 的 14 个输入及原 class 输出如下：

```text
literalInts:array=[1, 2, 3, -4]:trace=
emptyInts:array=[]:trace=
literalStrings:array=[left, null, right]:trace=
emptyStrings:array=[]:trace=
effectfulInts-1:array=[10, 15, 23]:trace=abc
effectfulInts0:exception=java.lang.ArithmeticException:message=/ by zero:trace=a
effectfulInts1:exception=java.lang.ArithmeticException:message=/ by zero:trace=ab
effectfulInts2:exception=java.lang.ArithmeticException:message=/ by zero:trace=abc
effectfulInts3:array=[-3, 5, 10]:trace=abc
effectfulStrings-1:array=[s010, s15, s23]:trace=ABC
effectfulStrings0:exception=java.lang.ArithmeticException:message=/ by zero:trace=A
effectfulStrings1:exception=java.lang.ArithmeticException:message=/ by zero:trace=AB
effectfulStrings2:exception=java.lang.ArithmeticException:message=/ by zero:trace=ABC
effectfulStrings3:array=[s0-3, s1-5, s2-10]:trace=ABC
```

原 class 和 JADX 全类恢复版本都以 `javac --release 8` 编译成功，并在 `java -Xverify:all` 下输出上述逐字节相同的 14 行；归一化输出 SHA-256 为 `f16eb426dc781c7ac365ff0f85b20c1452269145a306297802105f5eeb5a7349`。冻结 jarde 版本当前产生四个非空初始化方法缺少 `return` 的真实整类 javac 编译错误，位置对应 `literalInts`、`literalStrings`、`effectfulInts`、`effectfulStrings`；其结果不可运行。详细命令、原始输出、错误和 JADX 对照见证据目录的 `README.md`、`audit.json` 及 `jarde-javac.stderr`。

## 在现有测试体系中引用

根目录集成测试可用 `include_bytes!("fixtures/p3-array-initializers/v8/ArrayInitializerProbe.class")` 将原始 Java 8 class 交给现有恢复入口；crate 集成测试从 crate 目录使用对应相对路径 `include_bytes!("../../../tests/fixtures/p3-array-initializers/v8/ArrayInitializerProbe.class")`。完整类执行对照可复用 `tests/p3_execution_comparison.rs` 的 fixture case/harness；由 `ArrayInitializerProbe.java` 派生源码并让 Java 8 编译器与 runner 在临时目录执行候选类，runner 源码可通过 `include_str!("fixtures/p3-array-initializers/ArrayInitializerRunner.java")` 读取。成员正文、拒绝引用及 BCI/source map 断言放在数组恢复集成测试中。测试只读取已固定的 class/source，不应覆盖 fixture。

## 复现

```sh
cd tests/fixtures/p3-array-initializers
mkdir -p /tmp/jarde-array-initializers-repro
javac --release 8 -g:none -d /tmp/jarde-array-initializers-repro ArrayInitializerProbe.java
shasum -a 256 /tmp/jarde-array-initializers-repro/ArrayInitializerProbe.class
# 998bdb54c863d92cb63cc08c654df21bc674961c652fc46a5c3eaf7de2915c86
javac --release 8 -g:none -cp /tmp/jarde-array-initializers-repro -d /tmp/jarde-array-initializers-repro ArrayInitializerRunner.java
java -Xverify:all -cp /tmp/jarde-array-initializers-repro ArrayInitializerRunner
```
