# Lambda descriptor 边界扩展组

本目录以 Java 8 源码编译扩展真实 method-reference 形状，helper 与 runner 只是证据输入，不提交由它们编译出的 class。目录保存原 class `javap -v`、原/JADX/Jarde 完整类源码、每阶段编译与 JVM 状态、输出、class/source hash，以及本轮 Jarde CLI 的输入和结束 hash。使用 `python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/run_audit.py` 重放。

原 class 为 3728 bytes、21 个 methods/Code attributes、13 个 bootstrap entries，SHA-256 为 `592347dcc9201756c6bda19edf0517052bff41ff7640fceff79ebf892ac4e49a`。原类和 JADX 完整类在 `javac --release 8` 及 `java -Xverify:all` 下成功，输出逐项一致。Jarde 完整类在 `javac` 阶段失败，因此没有 Jarde JVM 输出；summary 中每项的 `failure_stage` 都指向这次完整类编译失败，不代表逐项运行后的差异。

这一原类包含 primitive implementation 返回到 boxed `Function<Integer,Integer>` 的真实 boxing bootstrap 和 `boundNull()`。后者在创建 bound method reference 时抛 `NullPointerException`。它也包含 `Supplier<String>` 的窄化返回目标。更聚焦的 raw/typed 返回用例及 runner `checkcast` 在 `return-probe/`。

`unknown-reference-probe/` 是单独标注的手工 invokedynamic 负对照：不相关接口类型被 LambdaMetafactory 拒绝。该负对照不是 javac 源类、不能算正例，也不能把其 bootstrap 失败归为恢复器偏差。
