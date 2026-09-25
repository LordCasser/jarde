## `p3-bitwise` fixture

本次只固定一份正面 Java 8 class：`BitwiseProbe.class`。`BitwiseEffects.java` 与
`BitwiseProbeRunner.java` 是 source-only helper/runner，不进入正面 class。正面 class 合并
了基础位运算和 type-flow 审计中已由 `javac --release 8 -g:none` 生成的形状：

- `int`/`long` 的 `&`、`|`、`^`、补码表达式、嵌套分组与 `-1` 边界；
- `boolean` 的 eager `&`、`|`、`^`，`true` 异或、条件消费，以及左右调用的顺序和抛错；
- boolean 嵌套、复制局部、分支内覆盖的 hoisted local、boolean 调用实参、boolean 数组元素；
- `byte`/`char` 提升到 `int` 的位运算；纯整数局部 `1`/`0` 经 `&`/`^` 组合，验证整数字面值不会自行启动 boolean 推断。

fixture 命令：

```text
javac --release 8 -g:none -d <temporary-classes> \
  BitwiseProbe.java BitwiseEffects.java BitwiseProbeRunner.java
java -Xverify:all -cp <temporary-classes> BitwiseProbeRunner
```

`BitwiseProbe.class` 为 major version 52，大小 1303 bytes，SHA-256 为
`d9cc8c8809e156ad2c1ba064dbda74dd1c7a6045cb7773d84a3ad40dd2874d9b`，含 23 个 `Code` 方法
（构造器加 22 个正面方法）。此前的 267 行覆盖仍保留；现在 runner 多执行一个
`integerLiteralControl()` 结果，共 268 行。测试会断言 `javac --release 8 -g:none` 从当前
源码重建的原 class 与冻结 class 字节完全一致，并在 `java -Xverify:all` 下执行。

关键物理 BCI（以 `javap -verbose` 为准）：

| 方法 | descriptor | 运算/消费 BCI |
| --- | --- | --- |
| `andInt` / `orInt` / `xorInt` | `(II)I` | `0,1,2,3` |
| `andLong` / `orLong` / `xorLong` | `(JJ)J` | `0,1,2,3` |
| `nested(int,int)` | `(II)I` | `0..9`，含 `ior=2,iand=7,ixor=8` |
| `complement` / `complementLong` | `(I)I` / `(J)J` | `ixor/lxor` 在 `2`/`4` |
| `andBoolean` / `orBoolean` / `xorBoolean` | `(ZZ)Z` | `0,1,2,3` |
| `constant` | `(Z)Z` | `0,1,2,3` |
| `ordered` | `(ZZ)Z` | `0,1,4,5,8,9` |
| `branch` | `(ZZ)I` | `ior=2, ifeq=3, returns `6/8/9/11` |
| `nested(boolean,boolean,boolean)` | `(ZZZ)Z` | `0..7` |
| `copied` | `(ZZZ)Z` | `0..15`，含三个局部转移 |
| `hoisted` | `(ZZZ)Z` | `0..13`，含条件覆盖 |
| `passed` | `(ZZZ)I` | `0..8`，位运算后调用 `accept` |
| `array` | `([Z)Z` | `0..7`，两个 `baload` 后 `ixor=6` |
| `promoted` | `(BC)I` | `0,1,2,3` |
| `integerLiteralControl` | `()I` | `0..9`，`iand=6,ixor=8,ireturn=9` |

## 三方基线

冻结 class 的完整输入在 `/tmp/jarde-bitwise-probe/` 做了三方对照：

- 原 class 与 source-only helper/runner 通过 `javac --release 8`，JVM 校验执行 267 行；
- JADX 完整 `BitwiseProbe.java` 与同一 helper/runner 通过 `javac --release 8`，
  `java -Xverify:all` 输出 267 行，与原 class 逐行一致；
- 当前未实施位运算生产的 jarde CLI 输出含 56 个 `@bytecode` 引用，完整恢复源以 `javac`
  失败（21 个正面运算方法均缺少返回语句），该红基线保存在临时
  `/tmp/jarde-bitwise-probe/jarde-javac.log`，不是测试对照的替代实现。

新增的 `integerLiteralControl` 是纯整数类型对照，不属于既有 JADX 267 行 oracle。整类实现测试使用
冻结原 class、当前完整 `Engine::class_source` 正文和同一 runner 比对 268 行；不声称此新增方法有
更新后的 JADX 输出。

原始 bitwise 审计的 211 行和 type-flow 审计的 56 行、mixed-types 合法 descriptor 负例及
JADX/jarde 失败记录分别保留在
`openspec/evidence/java-syntax-2026-09-22/bitwise/` 与其 `type-flow/`、`mixed-types/`
子目录；本 fixture 不把混合 boolean/int 变体复制成永久 class。
