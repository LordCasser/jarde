# EM-04：`package-info` 冻结对照

基线为本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 与 Jarde `54690b7f`。用 [replay.py](replay.py) 编译 [原始 Java 8 源码](../../../../tests/fixtures/proved-java-structure/package-info-basic/README.md)，核对两份冻结 class 的 SHA-256，随后将完整 jar 分别交给 JADX 和 Jarde，并用 `javac --release 8` 重编各自完整源码，最后以 `java -Xverify:all` 运行 `p.Check`。回放使用临时 Cargo target，结束时不保留编译残留。

| 来源 | `p/package-info.java` | 完整源码重编 | 运行 |
| --- | --- | --- | --- |
| 原始 | `@Deprecated` + `package p;` | 通过 | `true` |
| JADX | `@Deprecated` + `package p;` | 通过 | `true` |
| Jarde | `package p;` + `@java.lang.Deprecated` + `interface package-info {}` | 失败：`<identifier> expected` | 无法运行 |

冻结字节的 [javap](original-javap.txt) 表明这不是普通源码接口：`p/package-info`、版本 52、flags `0x1600`，无接口、字段和方法，只有一个运行时可见包注解。Jarde 既已读出并拼写注解，缺口在类级源码装配：把特殊物理类当成普通接口写出，并把 `package` 放在注解之前。

JADX 的 `TestPackageInfoSupport` 是 Smali 测试且禁用源码编译；其 `ClassNode.processSpecialClasses` 仅以简单名 `package-info` 和空成员判断特殊类，`ClassGen.makePackageInfo` 则在注解后写 `package` 行。这里借用后一个输出次序，但不能照搬宽松判定，否则非标准或证据不完整的空接口可能被误投影。首切片只接受完整证明的 Java 8 标准形状，复用 Jarde 现有 class-source 注解拼写与物理事实；不需要增加解析机制。实现合同见 [OpenSpec](../../../changes/recover-proved-package-info-source/)。

当前日志和两方源码快照记录于本目录；Jarde 的失败是预期基线，不能算一次运行通过。其余 EM-04 变体仍待扩验。
