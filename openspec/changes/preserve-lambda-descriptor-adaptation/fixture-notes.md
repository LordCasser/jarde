# Java 8 Lambda descriptor adaptation 证据

## 永久正例与完整类对照

永久 fixture 为 `tests/fixtures/p3-lambda-adaptation/v8/LambdaAdaptationProbe.class`。配套 helper 和 runner 只保留 Java 源码。冻结 class 为 2205 bytes、major version 52、9 个 methods/Code attributes、7 个 BootstrapMethods 项，SHA-256 为 `1f76c28361a2c54405e82950812863da1ea2f5a1012ef3a07e1049e5e4d45c6f`。`fixture/root/input-sha256.txt` 固定四个 Java 输入及这个 class 的哈希。

它保留 String/Object 重载、Object SAM 的动态 String→implementation Object 检查、String[]、primitive 同型、无捕获实例 receiver、constructor 和 raw generic return 控制。root replay 的完整源输入在 `openspec/evidence/java-syntax-2026-09-22/generic-method-references/fixture/root/inputs/`，原/JADX/Jarde 完整目标类源码、完整 runner、`javac` / JVM 状态、输出、`javap -v` 与 summary 均在同目录。可从仓库根重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/fixture/root/run_audit.py
```

该脚本以 `javac --release 8 -g:none` 从源文件重编译并要求字节与永久 class 完全相同，再以 `java -Xverify:all` 执行原类、JADX 类和 Jarde 类。27 个原类用例均成功执行，JADX 完整类编译并执行成功，27 行与原类一致。Jarde 完整类在 `javac` 阶段失败，所以这些用例没有 Jarde runtime 结果；summary 中的 `failure_stage: jarde-compile` 是完整类级失败，不能解释成每项单独执行后的差异。确切的三个错误是 String[] 动态参数恢复成 Object、无捕获 receiver lambda 的参数类型不兼容、constructor 不能把 Object 传给 String。它们是恢复输出失败；原 class 的 javac 与 JVM 状态均为 0，原 class 本身合法。

root replay 当时 `target/debug/jarde-cli` 的输入与结束 SHA-256 均为 `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`，记录在 `fixture/root/cli-hash-input.txt` 与 `cli-hash-end.txt`。这是那次 replay 的工具快照；后续 CLI 变更后要按脚本重新记录，不把此哈希当作当前工作树二进制哈希。

## Java 源码边界组

`openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/` 是 source-only 审计输入，脚本会重编译它并在 `summary.json` 中固定原 class hash：3728 bytes、21 个 methods/Code attributes、13 个 bootstrap entries，SHA-256 为 `592347dcc9201756c6bda19edf0517052bff41ff7640fceff79ebf892ac4e49a`。输入源 hash、Jarde CLI 输入/结束 hash 和三方完整类输出均保存于该目录。用以下命令重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/run_audit.py
```

这个 Java 8 原类把几个边界放在真实 bootstrap 与执行中：

| 边界 | 原 class 的真实证据 | 当前对照结论 |
| --- | --- | --- |
| bound-null | `boundNull()` 对 null receiver 创建 bound method reference，JVM 输出 `NullPointerException`；这是 factory 创建阶段失败。JADX 明确写出 `Objects.requireNonNull`。 | 原类合法并能执行。Jarde 完整源码在该方法只保留 `nullReceiver()` 调用，随后标出 invokedynamic/capture shape 未验证，没有返回函数对象；这属于未完成/拒绝输出，完整类也因其它方法不能编译，不能将其说成 bound-null 已成功恢复。 |
| primitive return → boxed SAM | `Function<Integer,Integer>` 引用返回 `int` 的实现方法；Bootstrap 参数给出 `(Integer)int` implementation 与 `(Object)Object` erased SAM，原 JVM 输出 `staticBoxedInt:15`。 | 这是实际合法的 boxing 形状。Jarde 完整类在其它方法处失败，故此组没有完整类执行对照；本项实现仍按现有证明边界处理 boxing，不以此手动构造 lambda 作为通过。 |
| 实际 SAM 返回窄化 | `Supplier<String>` 的 erased SAM 返回 `Object`，instantiated 返回 `String`；真实 implementation 为 `()Object`。见 `expanded/return-probe/`。 | raw `Supplier.get()` 返回 `Integer:7`；只有 typed caller 的 `checkcast String` 抛 CCE。原/JADX/Jarde 完整类在 String 与 Integer 两种状态的输出相同。此结果禁止把 instantiated `String` 误加成函数体返回检查。 |
| 无关引用关系 | 见下方手工 invokedynamic 负对照。 | 不属于 javac 合法的原源码形状，不是 Jarde 功能失败；用于证明未被证明的接口关系不能由“同为引用”推断。 |

扩展完整类的原/JADX `javac` 和 JVM 状态均为 0 且所有 21 项输出一致。Jarde 完整类 `javac` 失败，日志明确集中在 unbound instance 参数和 constructor 参数；没有 Jarde 运行结果。报告中的 class-wide `jarde-compile` 阶段不等于 21 项分别失败，也不能将整个类的编译失败归到 boxing、bound-null 或 return-probe。

focused return 重放命令是：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/return-probe/run_probe.py
```

输入与 CLI hash 分别保存在该目录。原 class 为 821 bytes、2 个 methods/Code attributes、1 个 bootstrap entry，SHA-256 为 `adfbe98a1409fd4b7cde9020ad66633f3f4bd2d3227e634d495721b81d1c1f3c`。`original-javap.txt` 保留实现 descriptor，`runner-javap.txt` 显示 `checkcast String` 在 typed caller 的 `Supplier.get()` 后。

## 未知接口关系负对照

`expanded/unknown-reference-probe/` 是显式标注的手工 classfile 对照，不是 javac 输出、正常原类或正面恢复例。用以下命令重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/unknown-reference-probe/run_probe.py
```

当前 JDK 自带 ASM 生成 version-52 字节码：same-type 的 instantiated/implementation 参数都是 `UnknownLeft`，执行返回 73 且调用一次；另一调用点使用无继承关系的 `UnknownRight` 与 `UnknownLeft`。即使值的具体类 `UnknownBoth` 同时实现两个接口，`LambdaMetafactory` 仍在函数对象创建时拒绝后一个调用点，输出 `BootstrapMethodError:calls=0`。`-Xverify:all` 已通过 classfile 验证，失败位置是首次 invokedynamic linkage。对应的普通 Java 方法引用也被 javac 拒绝。classfile、生成器、javap、控制编译诊断、运行输出和全部输入 hash 均在 probe 目录。

该手工负对照只支持这一条结论：当动态与实现引用类型没有已证明的可转换关系时，不应由恢复器凭引用槽形状补出检查或调用。它的 JVM bootstrap 失败不是原类合法执行后的恢复偏差，因此不计为恢复失败或正例。
