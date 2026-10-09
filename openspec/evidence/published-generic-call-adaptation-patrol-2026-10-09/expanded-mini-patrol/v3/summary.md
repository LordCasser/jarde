# 普通泛型调用适配：Corretto 8 debug 单腿探索

这轮仅作 exploratory 分类，未改生产代码、未跑 Cargo、未改 Git。基于已验收 CLI `/tmp/jarde-raw-receiver-final-v3-cli`（SHA-256 `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`），源码构建 JADX `/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`（版本 `dev`）。输入全部由 `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home` 的 javac 8.0.432 `-source 8 -target 8 -g` 从目录中的源文件编译，再打包成各自完整 jar。JADX 与 Jarde 都处理这些 jar；原始、JADX、Jarde 三类完整源分别在空 classpath 与空 sourcepath 下编译到临时隔离目录，运行时 classpath 只含新生成的 class，使用 `-Xverify:all`。所有临时编译 class 已清除，源码、jar、JADX 全量输出、Jarde 输出、完整 stdout/stderr 和 `manifest.json` 保留。

原源编译 10/10，JADX 整类重编与 marker 行为一致 10/10。Jarde CLI 对 10 个输入均返回 0，但整类重编只有 7/10 成功；成功的 7 类均通过 verifier 且 marker/选择结果与原源一致。另 3 类的真实 javac 诊断分别暴露 `Object[] -> T[]`、`Number -> T`、`Object -> T` 调用实参失败。未把 CLI 0 当作源码成功。

| 场景 | 原源方法头 | Jarde 输出方法头 | Jarde 整类结果 |
|---|---|---|---|
| `VoidDirect`：void sink(T) 直接调用 | `sink(T)`, `relay(T)` | `sink(T)`, `relay(T)` | 编译通过；marker 一致，唯一完整恢复 |
| `ArrayRelay`：T[] identity relay | `id(T[])`, `relay(T[])` | `id(T[])`, `relay(Object[])` | 失败：Object[] 不能传给 T[] |
| `NumberBoundRelay`：T extends Number relay | `id(T)`, `relay(T)` | `id(T)`, `relay(Number)` | 失败：Number 不能传给 T |
| `MethodShadow`：方法 T 遮蔽类 T，独立 U callee | `<T extends Number> relay(T)`, `<U extends Number> id(U)` | `relay(Number)`, `id(Number)` | 编译与 marker 通过；方法 binder 与 T 均反射擦除 |
| `IndependentCallee`：类 T 调独立 `<U> U id(U)` | `relay(T)`, `<U> id(U)` | `relay(Object)`, `id(Object)` | 编译与 marker 通过；调用链反射擦除 |
| `MultiParam`：两个 T 实参 | `first(T,T)`, `relay(T,T)` | `first(T,T)`, `relay(Object,Object)` | 失败：Object 不能传给 T |
| `RawReceiver`：raw List receiver | `relay(T)` | `relay(Object)` | 编译与 marker 通过；方法参数/返回反射擦除 |
| `SameNameOverload`：pick(T) 与 pick(String) | `pick(T)`, `pick(String)`, `relay(T)` | `pick(Object)`, `pick(String)`, `relay(T)` | 编译通过；运行仍选择 generic 分支，`pick(T)` 头被擦除 |
| `SameErasureBinder`：类 T 与独立 `<U extends Number & Runnable>`，首个擦除同为 Number | `relay(T)`, `<U extends Number & Runnable> sink(U)` | `relay(Number)`, `sink(Number)` | 编译与 null 行为通过；两个 binder 均反射擦除，身份不同未由同擦除证明 |
| `MutatedReceiver`：List<T> receiver 被 raw List 覆写后调用 | `relay(T)` | `relay(Object)` | 编译与 marker 通过；方法反射擦除 |

负控制 `IncompatibleBinder.java` 是刻意不合法的前置诊断：类 T 仅 extends Number，却将它传给要求 `U extends Number & Runnable` 的独立方法 binder。Corretto javac 返回 1，因此它没有 class/JAR，也不计入 10 个完整场景。

原字节码 javap 对照确认普通调用的物理目标：`VoidDirect.sink(Object)`、`ArrayRelay.id(Object[])`、`NumberBoundRelay.id(Number)`、`MethodShadow.id(Number)`、`IndependentCallee.id(Object)`、`MultiParam.first(Object,Object)`、`SameNameOverload.pick(Object)` 与 `SameErasureBinder.sink(Number)`。raw receiver 两项分别调用 `List.add(Object)` 和 `List.get(int)`；它们测试 raw/mutated 外部 receiver 的局部类型边界，不构成同类 callee 的恢复证明。每个完整 javap 保存于对应 `original-javap.stdout`，含 descriptor 与 code。

方法泛型头和 `GenericDeclaration` 反射输出由隔离 `HeaderProbe` 保存。原源与 JADX 的运行对照相同；Jarde 对成功重编样本的头差异已按表分类。所有命令的 argv、退出码以及 stdout/stderr SHA-256 在 `manifest.json`，输入源、jar、各类发射源、命令输出和 javap 也逐项存哈希。结果不是四腿验收：尚未跑 OpenJDK 23、no-debug，也未覆盖原 patrol 的构造器/EH样本；单腿结果不能代替四腿验收或据此批准泛型调用通用实现。
