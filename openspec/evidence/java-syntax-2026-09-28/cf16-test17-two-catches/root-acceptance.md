# Test17 四行双 catch/finally：root 独立验收

在主线 `ebb089eb` 上重新构建 `jarde-cli`，独立运行 `accept.sh` 到 `/private/tmp/jarde-cf16-test17-root-acceptance`。脚本核固定 JADX HEAD、原/探针 `test()I` 的逐 BCI/opcode/异常表，并将原类、固定 JADX、Jarde 的完整 Java 8 类分别重编、`java -Xverify:all` 运行。八条路径输出逐字一致；Jarde 目标方法只有一个 `finally`、两条具名 catch、一个提前 `return 1`，没有 explanation-only；source map 覆盖全部 18 个物理 BCI。

我核对了证书在既有 FINALLY pass 内独立接纳四行，物理覆盖限定为 `[0,3)` 的两具名 catch 加 catch-all，以及 `[16,19)` 的第二 catch 前缀 catch-all；五块 CFG 的正常与异常边均逐项核验。四份 `invokestatic ()V` 清理的同目标和 SSA 中的保存返回值、原 Throwable 重抛均有证明，AST 一次提交或回退。七个冻结变体分别改变调用目标、异常覆盖、行顺序、返回值、Throwable 或引入外部清理入口；它们都通过 JVM 验证、保留 `@bytecode` 而没有误发 `finally`。

同一新 CLI 的 Test16 `accept.sh` 六路径及五个近邻仍通过。`cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、OpenSpec strict 和差异空白检查通过。此验收只标记固定 Test17 的 Java 8 四行子形态；D8/DX 输出尚未重放，也不外推到其他双 catch 或 finally 布局。
