# DT-29：引用类型与接口强转

## 固定对象与判据

固定 inventory 是 `openspec/evidence/jadx-feature-inventory-2026-09-27/declarations-types.md` 的 DT-29，代表测试来自 JADX `types/TestInterfacesCast.java` 和 `types/TestFieldCast.java`。JADX checkout 固定在 `/Users/lordcasser/workspace/testzone/jadx` 的 `2fb1b16386941660fda07e9017285aec40fcb37f`，工作树干净。生产路径检查覆盖 `TypeInferenceVisitor` 对非 soft `CHECK_CAST` 建立 `TypeBoundCheckCastAssign`，以及 `InsnGen.makeInsnBody` 对 `CHECK_CAST`/`CAST` 输出 `(T) value`。

两个固定测试均无 `@NotYetImplemented`。接口测试关闭 debug info，并精确断言 `return (Runnable) closeable;`。字段测试关闭 debug info，要求 `((A) this)`、`((A) b)`、`((A) t)` 各出现一次，并排除 `unused =` 和访问修饰符变化。字段测试是组合形态：多个嵌套类、四种字段可见性、泛型边界及 private accessor。原复杂 fixture 和三方冻结输出留在 `combined/`，只作待扩验证据，不让它决定窄计划的通过条件。

## 隔离闭环一：接口 cast 与重载

`fixtures/interfaces/` 只含 `Closeable` 到 `Runnable` 的 cast、同一表达式触发 `Runnable` overload，以及一条不兼容对象路径。单独的 `InterfaceRunner` 对比对象身份、选中的 overload，并要求错误对象抛 `ClassCastException`。同一 runner 编译运行三方完整类源码：

| 输入 | `javac --release 8` | `java -Xverify:all` | 输出 |
| --- | ---: | ---: | --- |
| 原始源码 | 0 | 0 | `runnable:ClassCastException` |
| 固定 JADX | 0 | 0 | `runnable:ClassCastException` |
| Jarde | 0 | 0 | `runnable:ClassCastException` |

该闭环说明此接口 cast、运行时拒绝和 overload 形态在本切片中已追平；不外推到所有类型层次或所有消费位置。

## 隔离闭环二：最小 private 字段访问

`fixtures/private-field/PrivateFieldFamily.java` 只有一个根类、静态嵌套 `A`/`B`，以及一个 setter body。`B extends A`，一次调用分别写 `A.visible` public 字段和 `A.hidden` private 字段。外部 `PrivateFieldRunner` 用反射按固定 binary name 构造 `B`、调用 setter 并观察两个字段，因而原始/JADX/Jarde 都使用同一 Runner，且不会把源级嵌套拼写差异带入编译结果。

| 输入 | `javac --release 8` | `java -Xverify:all` | 输出/结果 |
| --- | ---: | ---: | --- |
| 原始源码 | 0 | 0 | `true:false` |
| 固定 JADX | 0 | 0 | `true:false` |
| Jarde | 1 | 未运行 | `PrivateFieldFamily$A.access$002` 缺少返回语句 |

Jarde 的重建源码和报告列出 setter 这条最小链上的三个失败点：

1. `B` 写入由父类 `A` 声明的 public 字段时，因 field owner 不属于 receiver `B` 自身声明而拒绝该写入。
2. private 写入变成 `A.access$002(A, boolean)` 调用，`B` receiver 到已证明父类 `A` 的实参转换没有获准。
3. `A.access$002` 中 javac 的 private 写入 helper 无法恢复为完整表达式/返回语句，重建类源码因此不能编译。

这三点都由一个 setter 和一个 synthetic accessor 暴露；夹具没有字符串拼接、泛型 setter、多层 child、`bits/run` 辅助方法或 synthetic 构造器。它们是一个窄字段写入闭环的前置子问题，修复目标不能只让 accessor 空体变得可编译而漏掉写入效果。

## 重放与范围

`replay.py` 固定验证 JADX commit/clean 状态，从两个夹具分别编译输入 class，打包每个完整类族，再生成 JADX/Jarde 的完整 class source。可编译结果用相同 Runner 经 `javac --release 8` 和 `java -Xverify:all` 对照。`outputs/` 保存输入 class 哈希、三方源码、Jarde JSON 报告、运行结果与稳定 SHA-256 清单。`combined/` 保存第一次较复杂的 `TestFieldCast` 风格夹具、固定 JADX 测试源和当时三方快照，供后续组合覆盖扩展时追溯。

本次 Jarde 基线为 `85117144069a5ab6901792b2c0a9f73951f2f3b8`，JDK 为 `23.0.1`。重建并重放：

```sh
CARGO_TARGET_DIR=/tmp/jarde-dt29-audit-target cargo build -p jarde-cli --bin jarde-cli
JARDE_CLI=/tmp/jarde-dt29-audit-target/debug/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/replay.py
```

只有隔离后的 private-field 闭环进入 [recover-private-field-owner-casts](../../../changes/recover-private-field-owner-casts/proposal.md)；本轮未改生产代码。复杂 `TestFieldCast` 的全类族组合仍待扩验。
