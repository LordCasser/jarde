# 未知接口关系负对照

本目录是手工生成 `invokedynamic` 的边界负对照，不是 javac 产物、正常 Java 方法引用或 Jarde 正面验收。它用于回答：当 instantiated 参数与 implementation 参数分别为无关接口时，能否假定存在运行时引用转换。

`GenerateUnknownReference.java` 使用当前 JDK 内置 ASM 生成 Java 8 classfile。classfile 的 erased SAM 参数为 `Object`，`sameType` 的 instantiated/implementation 参数均为 `UnknownLeft`；`unknownRelation` 则把 instantiated 参数设为 `UnknownRight`，implementation 参数仍为 `UnknownLeft`。两个接口没有继承关系，即使 `UnknownBoth` 同时实现二者，JVM 的 `LambdaMetafactory` 仍在创建函数对象时拒绝第二个调用点。

重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/unknown-reference-probe/run_probe.py
```

预期结果：same-type 控制返回 `73`、调用一次；未知接口关系在函数对象创建时抛 `BootstrapMethodError`、实现调用为零。`java -Xverify:all` 的类文件验证先通过，错误发生在首次 invokedynamic 链接。等价的普通 Java 方法引用也由 javac 拒绝。因本 classfile 是手工构造且原调用点会链接失败，不能把此处失败归因于恢复器，也不能把它当作可恢复的原始源码形状。

`input-sha256.txt` 固定所有源输入，`UnknownReferenceProbe.class` 与 `summary.json` 固定生成 classfile 哈希和运行结果，`original-javap.txt` 固定 bootstrap 参数。工具链版本及各阶段状态/输出均单独保存。
