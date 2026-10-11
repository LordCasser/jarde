# CF11 TestNestedLoops2：固定测试三方基线

Root 实际完成 29 命令 / 58 原始流，并独立检查通过。原始类、JADX 1.5.6 default/rename-none、Jarde default/all 的完整源码均在 JDK 8/23 编译并以 -Xverify:all 运行；九行行为同原始程序。原方法 j 是最大字符串长度，不是长度总和。异常输入核准确异常类型。

产品 b155877ee9a3b1894b053e70ffbf1cbf0a9512fa；冻结 CLI v10 SHA256 09f4eac277ee9ddfd60a627f7b132877841f4859bb2c655fc21cb8d25ac197db。完整构建 receipt 与 4707 输入见 recover-proved-nested-loop-side-exits/results/final-local-evidence-root-v1。

两个独立质量差距：JADX 保留 indexed for 和 List<String>；Jarde 输出尾部更新的 while，并因 ordinary_generic_source_unproved 退为 raw List。没有发现这九行输入的行为差异。default/all 全文和 maps、所有方法身份、物理 BCI、Code envelope、UTF8 来源已核；BLAKE3 仅形状核验，不称独立重算。

当前 ForHeader 条件白名单拒绝 Invoke 是源码与 javap 推导的候选门槛，尚无同一次 IR 的实际首拒 trace；实现前必须诊断。复用既有 ForHeader/Builder，不先增加机制。泛型 body proof 是独立架构债务。research 保留当前静态分析与根审阅过的固定顺序记录；其中其他固定测试的 oracle 是静态准备，未运行。

本证据只接受该基线观察，不接受实现，不关闭整个 CF11 或增加 71/612 完成数。归档按字节复制，来源/大小/SHA 见 archive-inventory.json；原始 private 目录保留。
