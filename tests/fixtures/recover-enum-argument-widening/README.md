# `recover-enum-argument-widening` 的冻结 fixture

`EN`/`ENN`/`ENT`/`ENX` 的源与两条 javac 腿的 class 文件。腿：

```sh
javac --release 8 -Xlint:-options -d v8 *.java                       # javac 23.0.1
/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 *.java
```

## 锚

- `EN.flags`：`OB.flags` 锚的顶层级（`ENT` 顶层枚举 + `EnumSet.of(ENT.A, ENT.C)` + `retainAll` +
  `contains`）。**两行机制分工**：`java.lang.Enum` 位由快照单边证明（`ENT`/`ENT$Flag` 的 class-file
  header 逐字写着 `java/lang/Enum`）——移除快照内的枚举类后该行逐字回归（`--policy single-class`
  实测）；本 change 的表承担 `EnumSet presents java.util.Collection`（BCI 12）——枚举族的
  `java.util.EnumSet` 自身 javadoc 行（extends `AbstractSet`；implemented-interface 列表达
  `Set`/`Collection`/`Iterable`）。
- `EN.three`：`EnumSet.of(E, E...)` 变长形（三常量）。
- `EN.same`：真 `java.lang.Enum` 声明型实参（类型变量擦除）——调用点不引入 cast（spec 第三 scenario）。
- `EN.eq`/`EN.h`/`EN.str`：`Objects.*` 零漂移对照（逐字等于 objects-enumset 巡查记录的健康面）。
- `ENN.flags`：巡查锚的 `$` 嵌套枚举形（`ENN$Flag`，池形拼写）。
- `ENX.platform`:平台枚举（`java.lang.Thread$State`）——不在快照 header、不入表，仍拒。

## 复现的既有边界（如实记录，非本 change 判据）

- `ENN` 的**类文本 fold** 两腿不同：javac 23 腿把嵌套枚举的声明发布进类文本并把引用重拼为
  `Flag.A`，真 javac 8 腿保持池形且无声明（既有 nested-enum 投影差异，本 change 只影响实参扩宽，
  与该差异无关）。故 `ENN` 只作逐字文本锚，不进 replay。
- `ENT` 自身的渲染不是合法源码单元（枚举类声明 `super(name, ordinal)`，源码不可写），replay 的
  `ENT.java` 用 fixture 自身声明；`EN` 的方法文本是渲染文本。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/EN.class` | `8a0ef939ac1e121cf30a032b8917e7bb99ee4cffa6efb9b965571f36747efe29` |
| `v8/ENN.class` | `d68934388e0ff6e37ba1b2ef76c06517eaa72810d02d45bb9b3e313513e1c721` |
| `v8/ENN$Flag.class` | `40d1ca3a7ee57cc27fd585d7d5f7bb898f8a0ed41b2f537b3d26a30673a02d0b` |
| `v8/ENT.class` | `a4dc4e077fd21774c5e44945284cd708fc25b917dec32e16593ec786ffbcb2d9` |
| `v8/ENX.class` | `e3999bcf1a419545de36960e6b1d531ca307889975fffa35891a6db9f9b6acc4` |
| `v8-javac8/EN.class` | `54d38e7a83691bf58a156cbd10fff635a274247b56012f2b1d620e8765744d4f` |
| `v8-javac8/ENN.class` | `a9779e5881a65a41b0ac6d24e58c781c8c262c16f494fa7418c9bcb54ea2b99b` |
| `v8-javac8/ENN$Flag.class` | `d21e27f23b13a79dad5a020fdf9a3fe5f51b9e0c8d023c07ca7c884c915f69c1` |
| `v8-javac8/ENT.class` | `dda146ec4736f8b8283045ce377d34813e16c532a2d015f340c0d4d60fa69bbe` |
| `v8-javac8/ENX.class` | `aa584709f0574166c0644953f5dca66c046daf510bbbdc1706f4b9a788d208c8` |

行为（replay 与 fixture 自身一致）：`EN` 的 `hasA/no/true/true/0/dflt`；`ENN` 的 `hasA`。
