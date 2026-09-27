## 1. 完整证据与受限发射

- [ ] 1.1 在 `jarde-java` 证明 exact standard-wrapper static `valueOf(primitive)` 的完整 descriptor、primitive literal、唯一直接 return 消费者，以及方法返回类型严格为对应 wrapper 或 `Object`；用六个 wrapper 正例和 owner/name/descriptor、consumer、非字面量、返回目标近似负例验证拒绝边界。
- [ ] 1.2 复用既有表达式/return 发射 primitive literal，保留 `byte`/`short` 窄化 cast，并把调用 BCI 保留为来源；以来源映射测试确认 literal、调用与 return 的物理位置均有归属，不新增通用转换 AST。

## 2. 编译运行与集成验收

- [ ] 2.1 扩展 EM-25 冻结回放：原/JADX/Jarde 完整类与共同 Runner 通过 `javac --release 8`、`java -Xverify:all`；检查六种包装类型、primitive 值及缓存范围内 identity 一致，且 `.longValue()`/null 拆箱行为维持原状。
- [ ] 2.2 对 proof 缺失形态验证保留显式 `valueOf` 或原拒绝；固定回放重跑两次并比较源、运行、CLI 与输入 SHA，确认 deterministic。
- [ ] 2.3 跑定向 builder/class-source 测试、`cargo fmt --all -- --check`、workspace check、`git diff --check` 和 `openspec validate debox-proved-wrapper-return-boxing --strict`；确认只改普通直接返回，不触及 EM-11 overload cast 或 DT-28 cast 规则。
