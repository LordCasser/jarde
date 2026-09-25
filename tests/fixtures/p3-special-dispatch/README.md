# P3 fixture：特殊调用保留选择语义

这个 fixture 固定 `invokespecial` 的三种接收者事实：直接父类、直接父接口和当前类的 private 方法。`SpecialProbe.value()` 的父类实现返回 7，因此覆写结果是 8；`DefaultProbe.super.value()` 返回 11；两个 private helper 用不同实例的 `bias` 区分 `this` 与参数接收者。参数求值还覆盖父类提供的副作用生产者、参数抛异常和父类方法抛异常，driver 覆盖 `null` 的其它实例接收者。

源码是 `BaseProbe.java`、`DefaultProbe.java`、`SpecialProbe.java` 和 `SpecialRunner.java`。使用 OpenJDK `javac 23.0.1` 生成三个被恢复的 Java 8 class：

```text
javac --release 8 -g:none -d v8 BaseProbe.java DefaultProbe.java SpecialProbe.java
```

编译器会输出 Java 8 选项过时警告，但 exit code 为 0。class 文件不含 debug attributes；编译器只在生成 fixture 时需要，测试运行时通过 `include_bytes!` 读取提交的字节。

`SpecialRunner.java` 只作为源码 driver 提交；运行对照时把它与匿名回调类一起编译到临时目录，不把 driver class 放进永久语料。

| class | version | bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `BaseProbe.class` | 52.0 | 833 | `b631eb77987041dd3d71a4e1098b08f2cddd613493a6a308add73ef1cdfe8266` |
| `DefaultProbe.class` | 52.0 | 114 | `de73401a2d112c9b10fdc19792388782117b47c4cc02716e34c820f6bb5b5711` |
| `SpecialProbe.class` | 52.0 | 784 | `a1cd50a4ae4b30779041521f50db5062cddd66ff067eff44328202f886479162` |
| `SpecialRunner.class` | — | — | 只提交源码；执行对照时在临时目录生成 |

当前 JVM 运行 driver 的事实输出为：

```text
value=8
defaultCall=11
own=7
other=8
otherNull=java.lang.NullPointerException
superSideEffect=8
sideEffectCount=1
superThrowingArgument=java.lang.IllegalArgumentException
superThrowingParent=java.lang.IllegalStateException
```

本 fixture 由 `tests/p3_special_dispatch.rs` 通过 `include_bytes!` 读取；后续 JDK ignored 对照必须从库生成的真实 class-source 文本编译，不能把修好的方法源码硬编码进 runner。

## 多个父接口 default 的证明控制

`InterfaceSuperDefaultConflict.java` 固定两个互不继承、都声明 `value()I` default 的父接口，以及为满足 Java 源码规则而覆写该方法的 `BothDefault` 子接口。使用 OpenJDK `javac` 编译时执行：

```text
javac --release 8 -g:none -d interface-super-proof InterfaceSuperDefaultConflict.java
```

`facade::interface_super_proof_tests::unique_source_default_rejects_multiple_inherited_defaults` 从 plain JAR 的选定定义读取这三个接口及其成员表，然后只在传给 `unique_source_default` 前从受控的 `BothDefault` facts 中移除源码为消解冲突所需的 override。这个变异只测试证明函数如何拒绝两个不相关的继承 default；它不声称变异后的 class 是 Java 源码合法定义，也不改变生产读取或证明实现。

| class | major version | bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `proof/LeftDefault.class` | 52 | 119 | `418e6271fad0c3e0133c603c357604f5ec30913e370f32237d6301b3fbda0265` |
| `proof/RightDefault.class` | 52 | 120 | `21b22a396c7696033a97337b8249c578261e7ec3c2962ff33b1c5735efdf68e9` |
| `proof/BothDefault.class` | 52 | 170 | `a5012f2130685a86f3decb3a6750f75534a7dd0c06a1fe627e6fcb6bfb19069b` |
