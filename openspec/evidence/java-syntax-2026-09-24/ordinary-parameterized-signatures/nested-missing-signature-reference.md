# 普通方法 `Signature` 中缺失的嵌套类型

## 结果

这个 fixture 隔离的是源码与环境边界，不是泛型签名语法错误。先用 `javac --release 8` 编译 `probe.NestedMissingSignature.echo(List<Payload>)`，再只把 `echo` 方法 `Signature` 常量中的 `Lprobe/Payload;` 改成等长的 `Lprobe/Missing;`。JVM descriptor 保持为 `(Ljava/util/List;)Ljava/util/List;`，方法体仍是原样返回参数。变造没有修改 `-g` 生成的 `LocalVariableTypeTable` 中 `Payload` 条目；配对的 `-g:none` class 不含局部变量调试信息。

`javap -v` 确认方法 `Signature` 的参数和返回类型都写为 `List<probe.Missing>`，descriptor 仍为 raw `List`。两种调试信息设置下，`java -Xverify:all` 都能执行变造后的方法；即使 classpath 中没有 `probe.Missing`，外部 runner 仍输出 `execute=1`。原始 class 的反射结果是 `List<probe.Payload>`。在 classpath 中缺少 `probe.Missing` 时，对变造方法反射读取泛型参数或返回类型会抛出 `TypeNotPresentException`（由 `ClassNotFoundException` 引起）；提供 `probe.Missing` class 后，反射结果为 `List<probe.Missing>`。

JADX 1.5.6 无论只接收目标 class，还是接收同时包含引用类型的 JAR，都会输出 `List<Missing>`。缺少 `Missing` 时，输出源码无法编译；提供该 class 后，生成的 class 可以编译和执行，反射结果为 `List<probe.Missing>`。当前 Jarde 对 standalone class、缺少该类型的 plain JAR、包含该类型的 plain JAR 都输出 `List<probe.Missing>`，三份生成的源码字节完全相同。Jarde 的 `-g:none` 输出也包含 `List<probe.Missing>`（参数名为 `arg0`）；提供该 class 后源码可以编译、执行并通过反射读取，缺少该 class 时 `javac` 会拒绝编译。

另有一条独立对照：`probe.Payload` 直接出现在方法 descriptor 中，且没有 `Signature`。Jarde 将其拼写为 `probe.Payload`；缺少 Payload 时生成源码同样无法编译，提供 Payload 后则可以编译。因此，反编译源码编译失败本身不足以证明存在 Signature 特有缺陷：编译环境可能只是缺少源码引用的类型。另一方面，验证结果确实表明当前 parser/projection 会接受并拼写一个擦除兼容的嵌套类型名，而不解析该类型是否存在。

## 实现证明了什么，本实验没有证明什么

`parse_method_signature` 解析属性语法。`prove_method_signature_erasure` 检查方法级类型变量作用域，并将每个顶层参数/返回类型的擦除与物理 descriptor 比较。对于 class 类型，`erase_type` 取最后一个二进制名称；它不会加载该 class。`class_source` 中的 `project_method_signature` 调用这些证明，然后由 `spell_ordinary_signature_type` 递归拼写类名和类型实参。这个拼写路径没有接收 resolution environment 参数。Jarde 的观察结果符合这一边界：把 `probe.Missing` 加入 plain 输入 JAR，不会改变生成源码。

这**不**表示该 `Signature` 会被 JVM 拒绝或对 JVM 无效，也不表示真实项目的 classpath 一定缺少其中所有嵌套引用。JVM verifier 接受并执行该 class；Java 反射延迟解析泛型类型，只有在需要具体化缺失的类型实参时才失败。受控的 `javac` 结果只说明每个特意选定的编译 classpath 是否包含 `probe.Missing` 或 `probe.Payload`。

面向任务的 Jarde CLI 每次打开一个 `--input` artifact，并且只把该 snapshot 绑定到 class-source 调用。本次重放对比 standalone 目标 class，以及包含或不包含引用类型的 JAR；没有通过该 CLI adapter 再提供第二个独立 artifact snapshot。因此，通过底层多 snapshot 调用单独提供一个 class 不在本次测试范围内。artifact 中是否包含该类型的对比足以表明：当选定的 plain-JAR root 中包含引用类型时，当前 projection 的输出仍不变；但这不是对所有显式 classpath 或 provider 的结论。

## 重放

在仓库根目录运行：

```sh
sh openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/replay_nested_missing.sh
```

脚本分别以 `-g` 和 `-g:none` 编译 Java 8 class 文件，只在方法的 `Signature` 属性处执行等长替换，然后运行 `javap`、JVM 验证/执行/反射、JADX、当前 Jarde CLI，以及源码编译和运行对照。脚本使用临时 `CARGO_TARGET_DIR`，退出时删除整个临时工作区（包括该 Cargo target）。记录的工具版本为：OpenJDK/Javac 23.0.1、JADX 1.5.6、Cargo 1.98.1。

记录本次运行的源码和输出 SHA-256：

| 产物 | SHA-256 |
|---|---|
| `NestedMissingSignature.java` | `335596e195e4bd9790b7b1b9aaa5b380bff8ca052c59acec286d4fc9598bff88` |
| `Payload.java` | `48c011aa7b9e16d676d058de2760433f4346fe9df60d8f5fd05a064ea577504c` |
| `DirectMissingControl.java` | `4b6bab936d7dedaa903b85144c18137073b22d826396b34b9b4d4588444f620b` |
| `NestedSignatureRunner.java` | `9ea0841bb206841c282a8cbf667459955e250d807c3b4dcf71ee68f96186d8d7` |
| `Missing.java` | `d6a4e701b231fed34af0e95f9fc2cc05101a81624e3f22a9ebab29b59a0efc27` |
| `patch_nested_signature.py` | `e0a5a2d81f8bf1458d6d51c1ed07ac294cb102385a72123806a18ee46e55264` |
| `replay_nested_missing.sh` | `bfad9b44bc856cf71954357a65bfe5ff481c5acb046f8f87f3d38d4774e7811d` |
| 原始 `-g` class | `7af73c772d0c5756358ba2c304900840906ff2efa35ad95397868eade4854166` |
| 变造 `-g` class | `ecab7255b55fca4b986fa79ed3dc1f67f53f29a2b86cc0f8ae1b1f29036a3a67` |
| 原始 `-g:none` class | `f01aeee2f6314c979ec62be2e33af34b4e3b0acf83260cf2973897a22ec13307` |
| 变造 `-g:none` class | `92477b4f1759a9ecb8898999634d7dccd0b576beab3fb92a5e4d797d55eaad25` |
| JADX 源码，仅目标 class 输入 | `3aee6ad8c800eed168d7d7e9008b0e55833b9994879eb0d095784ff647d6a2cd` |
| JADX 源码，输入 JAR 包含引用类型 | `687dc8b1657e98d1b04f8bf114f76ac6ba2e4df896a598791df82942271cc1f3` |
| Jarde 源码，`-g` | `cf048822ce32a5e681e4a41c92fcaa450e1619cf1b21bc1cc2b0e6a7626832ec` |
| Jarde 源码，`-g:none` | `59bcdbd54ca8ed3579e55a5ded4c736b0bea971086a2bd15f8a1b786086f5d20` |

本次 Jarde binary 基于工作树构建，HEAD 为 `a83b5572a93dd316dac1eb7697208fecd6a6fdbc`；当时工作树还包含其他并行的未提交改动。本次涉及的源码摘要为：`src/class_source.rs` `946dbdb2b1715bee874d9d3e5eda586ca29f64834ec4d1df8910288f074b78d3`，`crates/jarde-reader/src/signature.rs` `dff4cd23d2cda509ca30aabbbf89c37677d1e55a8bff7031fdb8fc28473449f9`，`crates/jarde-cli/src/task.rs` `c903074617e65984813d2a62d0433fd482600dace14ad84912e5e4dad0694ab6`。
