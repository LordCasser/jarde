# EM-25 主线独立验收

root 审阅实现并将 `c65df6c8` 合入主线后，重新构建 CLI（SHA-256 `aa5c292a9b376a7fb9759ce3c9fe71dab2971fb8d530840055353586f806f9ff`），独立运行固定 [replay.py](replay.py)。固定 JADX 提交及六个测试、四个生产文件的哈希通过核验。原 class、JADX、Jarde 的完整 `BoxingAudit` 源码均以 Java 8 重编，通过 `java -Xverify:all`，14 行输出逐字一致；独立结果保存在 [root-replay/](root-replay/)。

代码只在 `return` 直接且唯一消费精确 `Boolean.valueOf(Z)`、`Integer.valueOf(I)`、`Character.valueOf(C)`，实参又是唯一消费的整数常量时简写；布尔值限定 0/1，整数限定 -128..127，字符限定 ASCII 0..127，返回类型限定相应包装类或 `Object`。JLS 自动装箱与这三个标准包装类的缓存保证共同覆盖上述引用身份。Byte/Short/Long、越界值、`Number` 返回、嵌套调用与第二消费者均保留调用或物理回退；第二消费者另有单独生成源码和断言。字面量直接 BCI、省略调用的派生 BCI 和返回语句 BCI 均在同次 source map 中核对，预算检查在证明期间传播停止。

主线 `cargo test -p jarde-java`（223 个单元测试及集成测试）、`cargo test -p jarde-cli --test class_source_cli`（17 项）、`cargo check --workspace`、`cargo fmt --all -- --check` 与 `openspec validate debox-proved-wrapper-return-boxing --strict` 均通过。EM-25 仅此有界源码质量差距修复；其它拆箱、强转及重载形态继续按各自单元扩验。
