# 父类链造成的接口 `super` 冗余

`Probe.java` 和 `TransitiveProbe.java` 都以 `javac --release 8` 的 `-g` 与 `-g:none` 编译。两者的合法原源码从 `Child` 调用 `B.super.m()`，原 class 运行输出 `2`。前者由直接父类 `Base implements A`；后者由祖先 `Root implements A`，而 `Base extends Root` 本身没有接口行。`Child` 在两个样例里均直接实现 `A, B`。

`replay.py` 复用 [直接接口反例](../redundant-interface-super/replay.py)的常量池解析器，只把 `Child.class` 中唯一 `CONSTANT_InterfaceMethodref B.m:()I` 的 `class_index` 改为现有 `CONSTANT_Class p/A` 索引；Code、声明、其它池项不变。[反汇编](patched-javap-g.txt)显示 BCI 1 现在是 `invokespecial InterfaceMethod p/A.m:()I`。两个 patched class 都通过 `java -Xverify:all` 并输出 `1`。具体 class 散列见 [classfile-sha256.txt](classfile-sha256.txt)。

把原源码同一处调用直接写作 `A.super.m()`，两种父类深度在 Java 8 均不能编译；javac 报“默认超级调用中的类型限定符 A 错误；冗余接口 A 已由 Base 扩展”（见 [直接父类诊断](illegal-javac-g.txt)和[祖先诊断](transitive-illegal-javac-g.txt)）。旧 Jarde 把 patched 直接父类场景的两个调用都写作 `p.A.super.m()`，并标为 recovered；[完整输出](baseline-jarde-Child-g.java)同样被 javac 拒绝（[日志](baseline-jarde-javac-g.txt)）。这是 Java 源级合法性与 JVM 验证的差异，不是调用目标不明确。

因此合法 `I.super` 的非冗余证明不能只检查其它直接接口的继承边，还必须在同一次选定环境里遍历当前类的整个父类链，以及各父类实现的接口闭包。任何中间定义缺失、歧义或预算停止都不能当成“未实现 I”。接口本身的调用不走 class 父类链；default 目标与抽象覆盖仍需单独证明。JADX 已在[正常双接口对照](../redundant-interface-super/analysis.md)表现出把显式接口选择写成父类 `super` 的错值，因此不采用其文本作真值。

重放：`python3 replay.py --jarde-cli /tmp/jarde-cli-audit-baseline`。基线 CLI SHA-256 为 `60c045b8f94d363950525692f5ce9e5f5cde6f78134806d75801cd0cae89a573`。本次 JDK 的 `javac/java` 为 23.0.1；所有 class/JAR 临时文件在脚本结束时删除。文件散列见 [SHA256SUMS.txt](SHA256SUMS.txt)。
