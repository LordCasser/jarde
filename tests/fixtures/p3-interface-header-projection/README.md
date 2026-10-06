# `recover-parameterized-interface-headers` 的接口腿 fixture

变更 `recover-parameterized-interface-headers`（接口三道门放行 + 并列接口证明器）。源在顶层，class
分两条 javac 腿；`api1/` 是各类**真正编译时**依赖的定义（只用于编译、**绝不入环境**），`api2/` 是环境里
那一份**矛盾**定义（`Arity` 拒绝格的成因）。构建脚本（可重跑）：
`openspec/changes/recover-parameterized-interface-headers/results/03-fixtures/build-fixtures.sh`。

```sh
javac --release 8 -Xlint:-options -classpath api1-out -d v8 *.java   # javac 23.0.1，api2 的 ArityApi.class 一并入 v8/
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -classpath api1-out -d v8-javac8 *.java
```

## 各格与期望终态

| 类 | 声明（自身 `Signature`） | 期望 |
| --- | --- | --- |
| `IfaceImpl` | `implements java.lang.Comparable<IfaceImpl>` | 类头投影 `implements java.lang.Comparable<IfaceImpl>`，`compareTo(Object)` 桥隐藏（javac 从该头重建） |
| `MultiIface` | `implements java.lang.Comparable<MultiIface>, java.lang.Runnable` | 参数化项投影、`Runnable` 保持裸拼写；桥隐藏 |
| `ErasedCall` | `implements java.lang.Comparable<ErasedCall>`，body 里经接口引用自调用 | 类头投影、桥隐藏；**该 body** 的擦除调用仍拼 `(java.lang.Object)` ⇒ 该 body 不可编（调用点实参重定域的债，见 change 证据 03-fixtures） |
| `Unresolved` | `implements MissingApi<Unresolved>`（环境不装 `MissingApi.class`） | 保持物理裸拼写 `implements MissingApi`；桥**保持可见** |
| `Arity` | `implements ArityApi<Arity>`（环境装的是 2 形参的 `ArityApi`） | 类头投影**拒绝**（定义与 `Signature` 自相矛盾） |
| `TypeUse` | `implements java.lang.@Mark Comparable<TypeUse>`（类型注解形） | 与实现前逐字相同：裸头 + 桥可见（自有头门 → 静默裸头） |
| `Mark` | `@Target(TYPE_USE) @Retention(CLASS) @interface` | 供 `TypeUse` 的 `RuntimeInvisibleTypeAnnotations` 形 |

`IfaceImpl`/`MultiIface`/`ErasedCall`/`Unresolved`/`Arity` 均**不含**调用自身擦除契约的 body（`ErasedCall`
是唯一的例外，且正是被记录的边界格）——经接口引用的调用由测试内的 source runner 发出。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/IfaceImpl.class` | `d642efab0f1fc1f280909cdb53598cde73f5214e5b1ade22f48c00f74cb2a332` |
| `v8/MultiIface.class` | `312750626a4605caef0930ca62616fe31b7963f8ef1986fa2d63da9bd0b6f06c` |
| `v8/ErasedCall.class` | `23fa43b69c23db9b4e8ebab0c96e4cd2673c541a2c97d125be75db8d7eb1d597` |
| `v8/Unresolved.class` | `e3fac0586af885709bf8c14189b0ef0480a363b4fa16e3c544b244c6060e8ba9` |
| `v8/Arity.class` | `4af42b0ff281adc15b84d4a00911d3ecef65b4749d191a7e91671570a36daeaa` |
| `v8/ArityApi.class`（api2，2 形参） | `2faea7e043fe207e370eb68187f6c6075db586dee8675c32d8a38b3a9a495f9a` |
| `v8/TypeUse.class` | `c034e9e4058269b30b8a131ca430fcc6d6f3673a813b13341c4fd1da765628de` |
| `v8/Mark.class` | `a5a93cc6773273c78b9a59dc0db64db25cb17e9c60d384408cbb3b864e8bc81a` |
| `v8-javac8/IfaceImpl.class` | `4c73894b1a07e7f4932d2d8e00c42a1b38c6defb9cb870ad7cdfd442b5c57353` |
| `v8-javac8/MultiIface.class` | `23b2525a4db070af272c5cdf49004c3cb4a01dc79573b2a977e1fac1703a765b` |
| `v8-javac8/ErasedCall.class` | `e64857a6f1deb7c2d28458df9f5fa50f64e9dcfba34c0ca8cae01f80336039e9` |
| `v8-javac8/Unresolved.class` | `7601b34d15ac089aff8d5dd11022a944c77095b3973945f4b1cf3357c5480a36` |
| `v8-javac8/Arity.class` | `6c8a7376ae7332be2348cde092370261465f3a7663b44b545aafcc40eed3b9eb` |
| `v8-javac8/ArityApi.class`（api2，2 形参） | `2b0f7e8abb88b44ec05c320413ef605ffbbe897a3e4c42257de0d368b4506817` |
| `v8-javac8/TypeUse.class` | `9cd493d40f35795177e956330b9213a565b59c03abe714e409e984382df7625c` |
| `v8-javac8/Mark.class` | `1397cc43fc6f65ee24cf7d4faece01591ef435ff4cd1f327e82090c7bfc04945` |

两条腿的字节不同（真 javac 8 自己写常量池与 `StackMapTable`），行为一致；测试默认用 `v8` 腿
（`--release 8`），锚的性质在两条腿上相同。
