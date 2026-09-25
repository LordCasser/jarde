# 同方法双分配点负例

`AnonymousDoubleSite.java` 是可用 `javac --release 8 -g` 编译的原始 Java 8 场景：`create(boolean)` 有两个不同匿名类使用点。`freeze.py` 只在编译后的调用者 `Code` 中，把第二个使用点的 `new` 和 `invokespecial <init>` 常量池索引改指第一个匿名类；两个构造器均为 `()V`。保存在目录中的 `.class` 是变异后冻结副本，`$2` 仍保留供验证元数据与未使用类处理，不参与两处分配。

运行 `python3 tests/fixtures/proved-java-structure/anonymous-double-site/freeze.py` 会重新生成同一组 class、在 `java -Xverify:all` 下检查 `sameClass=true` 并打印 SHA-256。此 fixture 是带有合法 Java 8 源码起点的**字节码负例**；源码未变异时 `sameClass=false`，不能把原始源码当作冻结 class 的等价源码。
