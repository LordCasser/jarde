# `recover-charsequence-argument-widening` 的冻结 fixture

`CS`/`CSX` 的源与两条 javac 腿的 class 文件。腿：

```sh
javac --release 8 -Xlint:-options -d v8 *.java                       # javac 23.0.1
/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 *.java
```

## 锚（`CS`，两条腿逐字一致）

- `join`：`String.join("-", xs)` —— **两个位点**：首参 `String → java.lang.CharSequence`（类行）与
  第 2 参 `String[] → java.lang.CharSequence[]`（同一事实的数组位置投影）。实测：只落类行时第 2 参
  仍在 BCI 3 被拒，故数组位是主锚恢复的必需伴随。
- `appender`：`Appendable.append(CharSequence)` 位（`java.lang.Appendable` 形参）。
- `joining`：`Collectors.joining(",")`（`java.util.stream` 方法，巡查记录的第 5 位点）。
- `useBoth`：多重界 `both("a","b")` —— 擦除首界 `java.io.Serializable` 位（同批落地的 Serializable 表）。
- `same`：同型对照（`String → String`，不引入 cast）。

## 负例（`CSX`，仍拒）

- `sealBuilder`：`StringBuilder → java.io.Serializable` —— 实现者事实上成立，但 change 只钉
  java.lang 九行闭集（String + 八装箱），表外一行保持拒绝；
- `viaSegment`：`javax.swing.text.Segment → java.lang.CharSequence` —— release 8 javadoc 的
  CharSequence 实现者列表也含 `javax.swing.text.Segment`（`javap` 实测见
  `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`），change 的封闭四行不含它，
  故仍拒;两行均为**保守**拒绝（少放行，绝不误放行）。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/CS.class` | `bdf1c0ef724c9e015da58672044753d4fd4d297994128bcbdf519c8178ff0d22` |
| `v8/CSX.class` | `d4fcaa65036b5795e41857357b497a40e547213c5e28b0b3fbc7c1956c99f407` |
| `v8-javac8/CS.class` | `7f4402f551f1b9424842e551f85234dc25e8ac264918594d482c9e4358074614` |
| `v8-javac8/CSX.class` | `72187e25bac6c9db74be2bb299bdc9950083d844a74e6daab36c5986fdfee4e2` |

行为（replay 与 fixture 自身一致）：`p-q/a,b,c/b/s`。
