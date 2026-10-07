# `recover-boxed-number-widening` 的冻结 fixture

`C8`（巡查 fixture 源**逐字节复制**）/`BN`（本片六行 + 变体）/`BNX`（本片负例）的源与两条 javac 腿的
class 文件。腿：

```sh
javac --release 8 -Xlint:-options -d v8 *.java                       # javac 23.0.1
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 *.java
```

## 锚（两条腿逐字一致）

- `C8.main`（巡查主锚）：`larger(3, 7)` 的 `Integer → java.lang.Number` 位（巡查记 BCI 63 拒绝），
  恢复为 `larger((java.lang.Number) java.lang.Integer.valueOf(3), (java.lang.Number) java.lang.Integer.valueOf(7))`；
  同类的 `useWitness`（显式见证）、`loopBuilder`（循环携带 builder）是巡查记录的健康面，逐字未动。
- `BN.main`：六行各一条调用点——`Integer`/`Long`/`Double`/`Float`/`Short`/`Byte` 依次落在擦除的
  `java.lang.Number` 形参；`pickSeq("x", "yy")` 是 `String → CharSequence` 变体（姊妹片
  `recover-charsequence-argument-widening` 的行，本片不动其呈现）；`same(Integer.valueOf(9))` 是同型
  控制（不引入 cast）。
- `BN.withParam`：**装箱形参**（非字面量）落同一位点，第二个实参位同样呈现。

## 负例（`BNX`，仍拒）

`BigDecimal` 与 `AtomicInteger` 都是 `java.lang.Number` 的真实子类，但不在本片钉的 java.lang 六行里
（反射核对见 `changes/recover-boxed-number-widening/results/probe/number-universe.out`），故两个实参位
保持拒绝逐字。`Boolean → Number` 这类位置 javac 源级不可产生（`Boolean` 不转换到 `Number`），钉在
`build.rs` 的单元测试里。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/C8.class` | `e1e79bc9cb118c55855bcc29dbbfbb85f0a05e80e7a22dfac3cdfecace2c577b` |
| `v8/BN.class` | `b1d4721e6da3598506444bd1ca09b7b7949fbff8efa6627046e42dfb6c739ac7` |
| `v8/BNX.class` | `0fcaf74b923a0d5e59aee1fee7c964cd71d240abf9be186c278a566bfcc59c8c` |
| `v8-javac8/C8.class` | `c5a18ebb08ba681d09a05feccf3d061c2d07b9a59442c05d8eb6d58a0a571c17` |
| `v8-javac8/BN.class` | `30ec046d114158f03f51a13b129d1bd0aa1b8b78686aa927031c3d837ee64a12` |
| `v8-javac8/BNX.class` | `85d6dbc80d6f1912a24b953bff043119e2d00ec0220d565dabf4e0f1180495c6` |

源：`C8.java` `96f0c48f4b96adfa9a8438112532b73b0220a0d3c9b38713a676d616f7cb9d5d`（与巡查
`fixture/C8.java` 相同）、`BN.java` `2558b3b5df4cdc405f23513e58e3e0f638b5ecaa7421db22a52a93476d47e52b`、
`BNX.java` `4bd0f23ff72a72dcf2720dc9007276939c5c21cdf59f70ed0a8a2f02d631930c`。

行为（`java -Xverify:all`，两条腿相同，replay 与 fixture 自身 class 一致）：`C8` = `x/1:2/7/eoeoeoe`、
`BN` = `7/7/7.5/7.5/7/7/yy/9/4`、`BNX` = `2/2`。
